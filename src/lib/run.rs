use crate::lib::config;
use crate::lib::config::Config;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use std::process;
use std::process::Command;

#[derive(Debug)]
pub enum Error {
    NodeNotFound(which::Error),
    SuiteDoesNotExist,
    NodeProcess(io::Error),
    CopyingCustomHarness(io::Error),
    WritingHarness(io::Error),
    CopyingExpectedOutput(io::Error),
    Runtime(process::Output),
    CannotFindExpectedOutput,
    WrongOutputProduced { actual: Vec<u8>, expected: Vec<u8> },
}

pub fn run(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    if !suite.join("elm.json").exists() {
        return Err(Error::SuiteDoesNotExist);
    }
    let harness_file_name = "harness.js";
    let node_exe = which::which(&config.node).map_err(Error::NodeNotFound)?;

    let out_file = out_dir.join(harness_file_name);
    fs::copy(suite.join(harness_file_name), &out_file)
        .map(|_| ())
        .or_else(|e| match e.kind() {
            io::ErrorKind::NotFound => fs::write(&out_file, config::DEFAULT_HARNESS)
                .map(|_| ())
                .map_err(Error::WritingHarness),
            _ => Err(Error::CopyingCustomHarness(e)),
        })?;

    let expected_output = {
        let mut data = Vec::new();
        File::open(suite.join("output.txt"))
            .map_err(|_| Error::CannotFindExpectedOutput)?
            .read_to_end(&mut data)
            .map(|_| ())
            .map_err(Error::CopyingExpectedOutput)?;
        data
    };

    let res = Command::new(node_exe)
        .arg(out_file)
        .output()
        .map_err(Error::NodeProcess)?;

    if !res.status.success() {
        return Err(Error::Runtime(res));
    }

    if res.stdout != expected_output {
        return Err(Error::WrongOutputProduced {
            actual: res.stdout,
            expected: expected_output,
        });
    }

    Ok(())
}
