use crate::lib::config::Config;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::process;
use std::process::Command;
use std::str;
use std::string;

#[derive(Debug)]
pub enum Error {
    NodeNotFound(which::Error),
    SuiteDoesNotExist,
    NodeProcess(io::Error),
    WritingHarness(io::Error),
    CopyingExpectedOutput(io::Error),
    Runtime(process::Output),
    CannotFindExpectedOutput,
    ExpectedOutputNotUtf8(string::FromUtf8Error),
    OutputProduced(process::Output),
}

pub fn run(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    if !suite.join("elm.json").exists() {
        return Err(Error::SuiteDoesNotExist);
    }
    let expected_output_path = suite.join("output.json");
    let node_exe = which::which(&config.node).map_err(Error::NodeNotFound)?;
    let out_file = out_dir.join("harness.js");

    let expected_output = {
        let mut data = Vec::new();
        File::open(expected_output_path)
            .map_err(|_| Error::CannotFindExpectedOutput)?
            .read_to_end(&mut data)
            .map_err(Error::CopyingExpectedOutput)?;
        String::from_utf8(data).map_err(Error::ExpectedOutputNotUtf8)?
    };

    fs::write(
        &out_file,
        format!(
            r#"
const {{ Elm }} = require('./elm.js');
const expectedOutput = JSON.parse(String.raw`{}`);
{}

module.exports(Elm, expectedOutput);
"#,
            &expected_output,
            str::from_utf8(include_bytes!("../../embed-assets/run.js"))
                .expect("Embedded js template should be valid utf8."),
        ),
    )
    .map(|_| ())
    .map_err(Error::WritingHarness)?;

    let res = Command::new(node_exe)
        .arg("--unhandled-rejections=strict")
        .arg(out_file)
        .output()
        .map_err(Error::NodeProcess)?;

    if !res.status.success() {
        return Err(Error::Runtime(res));
    }
    if !res.stdout.is_empty() {
        return Err(Error::OutputProduced(res));
    }
    let debug_stderr : &[u8] = b"Compiled in DEV mode. Follow the advice at https://elm-lang.org/0.19.1/optimize for better performance and smaller assets.\n";
    if !res.stderr.is_empty() && &res.stderr[..] != debug_stderr {
        return Err(Error::OutputProduced(res));
    }

    Ok(())
}
