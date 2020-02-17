use super::config::Config;
use super::formatting;
use log::debug;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::str;
use std::string;

#[derive(Debug)]
pub enum CompileError {
    CompilerNotFound(which::Error),
    Process(io::Error),
    Compiler(process::Output),
    CompilerStdErrNotEmpty(process::Output),
    ReadingTargets(io::Error),
    SuiteDoesNotExist,
    OutDirIsNotDir,
}

#[derive(Debug)]
pub enum RunError {
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

#[derive(Debug)]
pub enum CompileAndRunError {
    Compiler(super::run::CompileError),
    Runner(super::run::RunError),
}

#[derive(Debug)]
pub enum SuiteError<P> {
    SuiteNotExist,
    SuiteNotDir,
    SuiteNotElm,
    Failure {
        allowed: bool,
        outdir: OutDir<P>,
        reason: CompileAndRunError,
    },
    ExpectedFailure,
}

#[derive(Debug)]
pub enum OutDir<P> {
    Provided(P),
    Tempory(tempfile::TempDir),
    Persistent(PathBuf),
}

pub fn compile(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), CompileError> {
    if !out_dir.exists() {
        let _ = fs::create_dir(out_dir);
    } else if !out_dir.is_dir() {
        return Err(CompileError::OutDirIsNotDir);
    }
    if !suite.join("elm.json").exists() {
        return Err(CompileError::SuiteDoesNotExist);
    }
    let root_files = if let Ok(mut targets) = File::open(suite.join("targets.txt")) {
        let mut contents = String::new();
        targets
            .read_to_string(&mut contents)
            .map_err(CompileError::ReadingTargets)?;
        contents.split('\n').map(String::from).collect()
    } else {
        vec![String::from("Main.elm")]
    };
    let elm_compiler =
        which::which(&config.elm_compiler).map_err(CompileError::CompilerNotFound)?;
    let mut command = Command::new(elm_compiler);
    command.current_dir(suite);
    command.arg("make");
    command.args(root_files);
    command.args(config.args.iter());
    command.arg("--output");
    command.arg(
        fs::canonicalize(out_dir)
            .map_err(CompileError::Process)?
            .join("elm.js"),
    );

    debug!("Invoking compiler: {:?}", command);

    let res = command.output().map_err(CompileError::Process)?;

    if !res.status.success() {
        return Err(CompileError::Compiler(res));
    }

    if !res.stderr.is_empty() {
        return Err(CompileError::CompilerStdErrNotEmpty(res));
    }

    Ok(())
}

pub fn run(suite: &Path, out_dir: &Path, config: &Config) -> Result<(), RunError> {
    if !suite.join("elm.json").exists() {
        return Err(RunError::SuiteDoesNotExist);
    }
    let expected_output_path = suite.join("output.json");
    let node_exe = which::which(&config.node).map_err(RunError::NodeNotFound)?;
    let out_file = out_dir.join("harness.js");

    let expected_output = {
        let mut data = Vec::new();
        File::open(expected_output_path)
            .map_err(|_| RunError::CannotFindExpectedOutput)?
            .read_to_end(&mut data)
            .map_err(RunError::CopyingExpectedOutput)?;
        String::from_utf8(data).map_err(RunError::ExpectedOutputNotUtf8)?
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
    .map_err(RunError::WritingHarness)?;

    let res = Command::new(node_exe)
        .arg("--unhandled-rejections=strict")
        .arg(out_file)
        .output()
        .map_err(RunError::NodeProcess)?;

    if !res.status.success() {
        return Err(RunError::Runtime(res));
    }
    if !res.stdout.is_empty() {
        return Err(RunError::OutputProduced(res));
    }
    let possible_stderr = |mode| {
        format!(
            "Compiled in {} mode. Follow the advice at https://elm-lang.org/0.19.1/optimize for better performance and smaller assets.\n",
            mode
        ).into_bytes()
    };
    if !res.stderr.is_empty()
        && res.stderr[..] != *possible_stderr("DEV")
        && res.stderr[..] != *possible_stderr("DEBUG")
    {
        return Err(RunError::OutputProduced(res));
    }

    Ok(())
}

impl<P> OutDir<P> {
    pub fn path(&self) -> &Path
    where
        P: AsRef<Path>,
    {
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
        replace_with::replace_with(
            self,
            || Self::Persistent(PathBuf::new()),
            |od| {
                if let Self::Tempory(tempdir) = od {
                    Self::Persistent(tempdir.into_path())
                } else {
                    od
                }
            },
        )
    }
}

pub fn compile_and_run(
    suite: &Path,
    out_dir: &Path,
    config: &Config,
) -> Result<(), CompileAndRunError> {
    super::run::compile(&suite, &out_dir, &config).map_err(CompileAndRunError::Compiler)?;
    super::run::run(&suite, &out_dir, &config).map_err(CompileAndRunError::Runner)?;
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

    if let Err(CompileAndRunError::Runner(super::run::RunError::Runtime(_))) = run_result {
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
) -> impl Iterator<Item = (Ps, Result<(), SuiteError<&Path>>)> + 'a {
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
