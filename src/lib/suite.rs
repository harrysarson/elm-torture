use super::cli;
use super::config::Config;
use super::formatting;
use log::debug;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
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
    ExpectedOutputPathNotUtf8(PathBuf),
}

#[derive(Debug)]
pub enum CompileAndRunError<P> {
    SuiteNotExist,
    SuiteNotDir,
    SuiteNotElm,
    CompileFailure {
        allowed: bool,
        reason: super::suite::CompileError,
    },
    RunFailure {
        allowed: bool,
        outdir: OutDir<P>,
        reason: super::suite::RunError,
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
    if let Some(elm_home) = env::var_os("ELM_HOME") {
        command.env("ELM_HOME", elm_home);
    }
    command.arg(
        fs::canonicalize(out_dir)
            .map_err(CompileError::Process)?
            .join("elm.js"),
    );

    debug!("Invoking compiler: {:?}", command);

    let res = (|| {
        for _ in 0..(config.compiler_reruns.get() - 1) {
            let op = command.output().map_err(CompileError::Process)?;
            if op.status.success() {
                return Ok(op);
            }
        }
        command.output().map_err(CompileError::Process)
    })()?;

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
    let harness_file = out_dir.join("harness.js");
    let main_file = out_dir.join("main.js");

    let canonical_expected_output_path = expected_output_path
        .canonicalize()
        .map_err(|_| RunError::CannotFindExpectedOutput)?;

    fs::write(
        &harness_file,
        &include_bytes!("../../embed-assets/run.js")[..],
    )
    .map_err(RunError::WritingHarness)?;

    write!(
        &File::create(&main_file).map_err(RunError::WritingHarness)?,
        r#"
const {{ Elm }} = require('./elm.js');
const harness = require('./harness.js');
const expectedOutput = require('{}');

harness(Elm, expectedOutput);
"#,
        match canonical_expected_output_path.to_str() {
            Some(p) => p.replace('\\', "\\\\"),
            None =>
                return Err(RunError::ExpectedOutputPathNotUtf8(
                    canonical_expected_output_path
                )),
        }
    )
    .map_err(RunError::WritingHarness)?;

    let res = Command::new(node_exe)
        .arg("--unhandled-rejections=strict")
        .arg(&main_file)
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

pub fn compile_and_run<Ps: AsRef<Path>, Pe: AsRef<Path>>(
    suite: Ps,
    provided_out_dir: Option<Pe>,
    instructions: &super::cli::Instructions,
) -> Result<(), CompileAndRunError<Pe>> {
    if !suite.as_ref().exists() {
        return Err(CompileAndRunError::SuiteNotExist);
    }
    if !suite.as_ref().is_dir() {
        return Err(CompileAndRunError::SuiteNotDir);
    }
    if !suite.as_ref().join("elm.json").exists() {
        return Err(CompileAndRunError::SuiteNotElm);
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
    fs::remove_dir_all(suite.as_ref().join("elm-stuff"))
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .expect("Could not delete elm-stuff directory");

    let mut out_dir = if let Some(dir) = provided_out_dir {
        OutDir::Provided(dir)
    } else {
        let dir = tempfile::Builder::new()
            .prefix("elm-torture")
            .tempdir()
            .expect("Should be able to create a temp_file");
        OutDir::Tempory(dir)
    };

    super::suite::compile(suite.as_ref(), out_dir.path(), &instructions.config).map_err(|e| {
        CompileAndRunError::CompileFailure {
            allowed: failure_allowed,
            reason: e,
        }
    })?;

    super::suite::run(suite.as_ref(), out_dir.path(), &instructions.config).map_err(|e| {
        out_dir.persist();
        CompileAndRunError::RunFailure {
            allowed: failure_allowed,
            outdir: out_dir,
            reason: e,
        }
    })?;

    // if let Err(CompileAndRunError::Runner(super::suite::RunError::Runtime(_))) = run_result {
    //     out_dir.persist()
    // };

    if failure_allowed {
        Err(CompileAndRunError::ExpectedFailure)
    } else {
        Ok(())
    }
}

pub fn compile_and_run_suites<'a, Ps: AsRef<Path> + 'a>(
    suites: impl Iterator<Item = Ps> + 'a,
    instructions: &'a super::cli::Instructions,
) -> impl Iterator<Item = (Ps, Result<(), CompileAndRunError<&Path>>)> + 'a {
    suites
        .map(move |suite: Ps| {
            let res: Result<(), CompileAndRunError<&Path>> =
                compile_and_run(&suite, None, instructions);
            if let Err(ref e) = res {
                println!(
                    "{}",
                    formatting::compile_and_run_error(
                        e,
                        &suite,
                        match instructions.task {
                            cli::Task::RunSuite { ref out_dir, .. } => out_dir.as_ref(),
                            _ => None,
                        }
                    )
                );
            }
            let failed = match res {
                Err(CompileAndRunError::CompileFailure { allowed: true, .. })
                | Err(CompileAndRunError::RunFailure { allowed: true, .. })
                | Ok(_) => false,
                Err(_) => true,
            };
            ((suite, res), failed)
        })
        .take_while(move |(_, failed)| !(instructions.fail_fast && *failed))
        .map(|(tup, _)| tup)
}
