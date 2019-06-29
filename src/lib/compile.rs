
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
}

pub fn compile(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    let program = which::which(&config.program).map_err(Error::CompilerNotFound)?;

    let res = Command::new(program)
        .args(&["make", "src/Main.elm", "--output"])
        .current_dir(suite)
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
