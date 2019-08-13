pub mod compile;
pub mod config;
pub mod find_suites;
pub mod run;

use compile::compile;
use config::Config;
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
