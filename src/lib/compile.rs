use crate::lib::config::Config;
use std::io;
use std::path::Path;

use log::debug;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::process;
use std::process::Command;

#[derive(Debug)]
pub enum Error {
    CompilerNotFound(which::Error),
    Process(io::Error),
    Compiler(process::Output),
    CompilerStdErrNotEmpty(process::Output),
    ReadingTargets(io::Error),
    SuiteDoesNotExist,
    OutDirIsNotDir,
}

pub fn compile(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    if !out_dir.exists() {
        let _ = fs::create_dir(out_dir);
    } else if !out_dir.is_dir() {
        return Err(Error::OutDirIsNotDir);
    }
    if !suite.join("elm.json").exists() {
        return Err(Error::SuiteDoesNotExist);
    }
    let root_files = if let Ok(mut targets) = File::open(suite.join("targets.txt")) {
        let mut contents = String::new();
        targets
            .read_to_string(&mut contents)
            .map_err(Error::ReadingTargets)?;
        contents.split('\n').map(String::from).collect()
    } else {
        vec![String::from("Main.elm")]
    };
    let elm_compiler = which::which(&config.elm_compiler).map_err(Error::CompilerNotFound)?;
    let mut command = Command::new(elm_compiler);
    command.current_dir(suite);
    command.arg("make");
    command.args(root_files);
    command.arg("--output");
    command.arg(
        fs::canonicalize(out_dir)
            .map_err(Error::Process)?
            .join("elm.js"),
    );

    debug!("Invoking compiler: {:?}", command);

    let res = command.output().map_err(Error::Process)?;

    if !res.status.success() {
        return Err(Error::Compiler(res));
    }

    if !res.stderr.is_empty() {
        return Err(Error::CompilerStdErrNotEmpty(res));
    }

    Ok(())
}
