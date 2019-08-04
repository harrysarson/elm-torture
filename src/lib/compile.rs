use crate::lib::config::Config;
use std::io;
use std::path::Path;

use std::process;
use std::process::Command;

pub enum Error {
    CompilerNotFound(which::Error),
    ProcessError(io::Error),
    CompilerError(process::Output),
    CompilerStdErrNotEmpty(process::Output),
    SuiteDoesNotExist,
}

pub fn compile(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    if !suite.join("elm.json").exists() {
        return Err(Error::SuiteDoesNotExist);
    }
    let elm_compiler = which::which(&config.elm_compiler).map_err(Error::CompilerNotFound)?;
    let res = Command::new(elm_compiler)
        .current_dir(suite)
        .args(&["make", "src/Main.elm", "--output"])
        .arg(out_dir.join("elm.js"))
        .output()
        .map_err(Error::ProcessError)?;

    if !res.status.success() {
        return Err(Error::CompilerError(res));
    }

    if !res.stderr.is_empty() {
        return Err(Error::CompilerStdErrNotEmpty(res));
    }

    Ok(())
}
