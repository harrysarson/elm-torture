pub mod config;
pub mod compile;
pub mod run;
pub mod find_suites;

use config::Config;
use compile::compile;
use run::run;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    Compiler(compile::Error),
    Runner(run::Error),
}

pub fn run_suite(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    compile(&suite, &out_dir, &config).map_err(Error::Compiler)?;
    run(&suite, &out_dir, &config).map_err(Error::Runner)?;
    Ok(())
}