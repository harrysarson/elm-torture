use super::config::Config;
use crate::formatting;
use colored::Colorize;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Compiler(super::compile::Error),
    Runner(super::run::Error),
}

#[derive(Debug)]
pub enum OutDir<'a> {
    Provided(&'a Path),
    Tempory(tempfile::TempDir),
    Persistent(PathBuf),
}

impl<'a> OutDir<'a> {
    pub fn path(&self) -> &Path {
        match self {
            Self::Provided(p) => p,
            Self::Tempory(ref p) => p.path(),
            Self::Persistent(ref p) => p,
        }
    }

    pub fn is_tempory(&self) -> bool {
        match self {
            Self::Provided(_) => false,
            Self::Tempory(_) | Self::Persistent(_) => true,
        }
    }

    pub fn persist(&mut self) {
        // A juggle to drop the tempdir contained behind a mutable reference.
        if let OutDir::Tempory(_) = self {
            let dir = mem::replace(self, OutDir::Persistent(PathBuf::new()));
            if let OutDir::Tempory(tempdir) = dir {
                mem::replace(self, OutDir::Persistent(tempdir.into_path()));
            } else {
                panic!("Impossible state!");
            }
        }
    }
}
#[derive(Debug)]
pub struct SuiteFailure<'a> {
    pub suite: &'a Path,
    pub outdir: OutDir<'a>,
    pub reason: Error,
}

#[derive(Debug)]
pub enum SuiteError<'a> {
    SuiteNotExist(&'a Path),
    SuiteNotDir(&'a Path),
    SuiteNotElm(&'a Path),
    Failure {
        allowed: bool,
        reason: SuiteFailure<'a>,
    },
    ExpectedFailure,
}
pub fn compile_and_run(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    super::compile::compile(&suite, &out_dir, &config).map_err(Error::Compiler)?;
    super::run::run(&suite, &out_dir, &config).map_err(Error::Runner)?;
    Ok(())
}

pub fn compile_and_run_suite<'a>(
    suite: &'a Path,
    provided_out_dir: Option<&'a Path>,
    instructions: &super::cli::Instructions,
) -> Result<(), SuiteError<'a>> {
    if !suite.exists() {
        return Err(SuiteError::SuiteNotExist(suite));
    }
    if !suite.is_dir() {
        return Err(SuiteError::SuiteNotDir(suite));
    }
    if !suite.join("elm.json").exists() {
        return Err(SuiteError::SuiteNotElm(suite));
    }
    let failure_allowed = instructions.config.allowed_failures.iter().any(|p| {
        if p.exists() {
            same_file::is_same_file(suite, p).unwrap_or_else(|e| {
                panic!(
                    "Error when comparing the paths {:?} and {:?}: {:?}",
                    suite, p, e
                )
            })
        } else {
            false
        }
    });
    if instructions.clear_elm_stuff {
        fs::remove_dir_all(suite.join("elm-stuff"))
            .or_else(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .expect("Could not delete elm-stuff directory");
    }

    let mut out_dir = if let Some(dir) = provided_out_dir {
        OutDir::Provided(dir)
    } else {
        let dir = tempfile::Builder::new()
            .prefix("elm-torture")
            .tempdir()
            .expect("Should be able to create a temp_file");
        OutDir::Tempory(dir)
    };

    let run_result = compile_and_run(&suite, out_dir.path(), &instructions.config);

    if let Err(Error::Runner(super::run::Error::Runtime(_))) = run_result {
        out_dir.persist()
    };

    if failure_allowed && run_result.is_ok() {
        Err(SuiteError::ExpectedFailure)
    } else {
        run_result.map_err(|err| SuiteError::Failure {
            allowed: failure_allowed,
            reason: SuiteFailure {
                outdir: out_dir,
                suite,
                reason: err,
            },
        })
    }
}

pub fn compile_and_run_suites<'a>(
    welcome_message: &str,
    suites: &'a [PathBuf],
    instructions: &super::cli::Instructions,
) -> Vec<(&'a PathBuf, Result<(), SuiteError<'a>>)> {
    assert!(!suites.is_empty());

    println!(
        "{}

Running the following {} SSCCE{}:
{}
",
        welcome_message,
        suites.len(),
        if suites.len() == 1 { "" } else { "s" },
        indented::indented(formatting::easy_format(|f| {
            for path in suites.iter() {
                writeln!(f, "{}", path.display())?
            }
            Ok(())
        }))
    );

    let suite_results = {
        let mut tmp = Vec::with_capacity(suites.len());
        for suite in suites {
            let res = compile_and_run_suite(suite, None, instructions);
            if let Err(ref e) = res {
                println!("{}", e);
            }
            let failed = match res {
                Err(SuiteError::Failure { allowed: true, .. }) | Ok(()) => false,
                Err(_) => true,
            };
            tmp.push((suite, res));
            if instructions.fail_fast && failed {
                break;
            }
        }
        tmp
    };

    println!(
        "
elm-torture has run the following {} SSCCE{}:
{}
",
        suite_results.len(),
        if suite_results.len() == 1 { "" } else { "s" },
        indented::indented(formatting::easy_format(|f| {
            for (path, result) in &suite_results {
                writeln!(
                    f,
                    "{} ({})",
                    path.display(),
                    match result {
                        Err(SuiteError::Failure { allowed: true, .. }) =>
                            "allowed failure".yellow(),
                        Err(SuiteError::ExpectedFailure) => "success when failure expected".red(),
                        Err(_) => "failure".red(),
                        Ok(()) => "success".green(),
                    }
                )?
            }
            Ok(())
        }))
    );
    suite_results
}
