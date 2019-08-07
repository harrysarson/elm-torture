use crate::lib::config::Config;
use std::io;
use std::path::Path;

use std::process;
use std::process::Command;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum Error {
    CompilerNotFound(which::Error),
    Process(io::Error),
    Compiler(process::Output),
    CompilerStdErrNotEmpty(process::Output),
    ReadingTargets(io::Error),
    SuiteDoesNotExist,
}

pub fn compile(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    if !suite.join("elm.json").exists() {
        return Err(Error::SuiteDoesNotExist);
    }
    let root_files = if let Ok(mut targets) = File::open(suite.join("targets.txt")) {
        let mut contents = String::new();
        targets.read_to_string(&mut contents).map_err(Error::ReadingTargets)?;
        contents.split('\n').map(String::from).collect()
    } else {
        vec![String::from("src/Main.elm")]
    };
    let elm_compiler = which::which(&config.elm_compiler).map_err(Error::CompilerNotFound)?;
    let res = Command::new(elm_compiler)
        .current_dir(suite)
        .arg("make")
        .args(root_files)
        .arg("--output")
        .arg(out_dir.join("elm.js"))
        .output()
        .map_err(Error::Process)?;

    if !res.status.success() {
        return Err(Error::Compiler(res));
    }

    if !res.stderr.is_empty() {
        return Err(Error::CompilerStdErrNotEmpty(res));
    }

    Ok(())
}
