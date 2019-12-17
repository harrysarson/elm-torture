pub mod compile;
pub mod config;
pub mod find_suites;
pub mod run;

use compile::compile;
use config::Config;
use run::run;
use std::fmt;
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

#[derive(Debug)]
pub struct Formatable<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> {
    func: F,
}

impl<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> fmt::Display for Formatable<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.func)(f)
    }
}

pub fn easy_format<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result>(func: F) -> Formatable<F> {
    Formatable { func }
}
