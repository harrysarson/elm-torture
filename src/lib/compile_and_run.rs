use super::config::Config;
use super::formatting;
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
pub enum OutDir<P> {
    Provided(P),
    Tempory(tempfile::TempDir),
    Persistent(PathBuf),
}

impl<P: AsRef<Path>> OutDir<P> {
    pub fn path(&self) -> &Path {
        match self {
            Self::Provided(p) => p.as_ref(),
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
pub enum SuiteError<P> {
    SuiteNotExist,
    SuiteNotDir,
    SuiteNotElm,
    Failure {
        allowed: bool,
        outdir: OutDir<P>,
        reason: Error,
    },
    ExpectedFailure,
}

pub fn compile_and_run(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), Error> {
    super::compile::compile(&suite, &out_dir, &config).map_err(Error::Compiler)?;
    super::run::run(&suite, &out_dir, &config).map_err(Error::Runner)?;
    Ok(())
}

pub fn compile_and_run_suite<Ps: AsRef<Path>, Pe: AsRef<Path>>(
    suite: Ps,
    provided_out_dir: Option<Pe>,
    instructions: &super::cli::Instructions,
) -> Result<(), SuiteError<Pe>> {
    if !suite.as_ref().exists() {
        return Err(SuiteError::SuiteNotExist);
    }
    if !suite.as_ref().is_dir() {
        return Err(SuiteError::SuiteNotDir);
    }
    if !suite.as_ref().join("elm.json").exists() {
        return Err(SuiteError::SuiteNotElm);
    }
    let failure_allowed = instructions.config.allowed_failures.iter().any(|p| {
        if p.exists() {
            same_file::is_same_file(&suite, p).unwrap_or_else(|e| {
                panic!(
                    "Error when comparing the paths {:?} and {:?}: {:?}",
                    suite.as_ref(),
                    p,
                    e
                )
            })
        } else {
            false
        }
    });
    if instructions.clear_elm_stuff {
        fs::remove_dir_all(suite.as_ref().join("elm-stuff"))
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

    let run_result = compile_and_run(&suite.as_ref(), out_dir.path(), &instructions.config);

    if let Err(Error::Runner(super::run::Error::Runtime(_))) = run_result {
        out_dir.persist()
    };

    if failure_allowed && run_result.is_ok() {
        Err(SuiteError::ExpectedFailure)
    } else {
        run_result.map_err(|err| SuiteError::Failure {
            allowed: failure_allowed,
            outdir: out_dir,
            reason: err,
        })
    }
}

pub fn compile_and_run_suites<'a, Ps: AsRef<Path> + 'a>(
    suites: impl Iterator<Item = Ps> + 'a,
    instructions: &'a super::cli::Instructions,
) -> impl Iterator<Item = (Ps, Result<(), SuiteError<PathBuf>>)> + 'a {
    suites
        .map(move |suite: Ps| {
            let res = compile_and_run_suite(&suite, None, instructions);
            if let Err(ref e) = res {
                println!("{}", formatting::suite_error(e, &suite));
            }
            let failed = match res {
                Err(SuiteError::Failure { allowed: true, .. }) | Ok(_) => false,
                Err(_) => true,
            };
            ((suite, res), failed)
        })
        .take_while(move |(_, failed)| !(instructions.fail_fast && *failed))
        .map(|(tup, _)| tup)
}
