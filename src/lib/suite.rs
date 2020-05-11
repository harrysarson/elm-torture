use super::cli;
use super::config::Config;
use super::formatting;
use futures::future;
use futures::future::Either;
use futures::future::TryFutureExt;
use futures::stream::Stream;
use futures::stream::StreamExt;
use log::debug;
use process::{Output, Stdio};
use std::env;
use std::fs;
use std::fs::File;
use std::{
    io::Read,
    io::Write,
    path::Path,
    path::PathBuf,
    process, string,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{io, process::Command, time};

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

#[allow(dead_code)]
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
    Timeout {
        after: Duration,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
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

pub async fn compile(
    suite: &Path,
    out_dir: &Path,
    config: &Config<impl AsRef<Path>>,
) -> Result<(), CompileError> {
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
        which::which(&config.elm_compiler()).map_err(CompileError::CompilerNotFound)?;
    let mut command = Command::new(elm_compiler);

    command.current_dir(suite);
    command.arg("make");
    command.args(root_files);
    command.args(config.args().iter());
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

    let mut compile = || {
        fs::remove_dir_all(suite.join("elm-stuff"))
            .or_else(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .expect("Could not delete elm-stuff directory");
        command.output().map_err(CompileError::Process)
    };

    let res = async {
        for _ in 0..(config.compiler_reruns().get() - 1) {
            let op = compile().await?;
            if op.status.success() {
                return Ok(op);
            }
        }
        compile().await
    }
    .await?;

    if !res.status.success() {
        return Err(CompileError::Compiler(res));
    }

    if !res.stderr.is_empty() {
        return Err(CompileError::CompilerStdErrNotEmpty(res));
    }

    Ok(())
}

pub async fn run(
    suite: &Path,
    out_dir: &Path,
    config: &Config<impl AsRef<Path>>,
) -> Result<(), RunError> {
    async fn read_to_buf(mut read: impl io::AsyncRead + Unpin) -> io::Result<Vec<u8>> {
        use io::AsyncReadExt;
        let mut buffer = Vec::new();

        // read the whole file
        read.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    if !suite.join("elm.json").exists() {
        return Err(RunError::SuiteDoesNotExist);
    }
    let expected_output_path = suite.join("output.json");
    let node_exe = which::which(&config.node()).map_err(RunError::NodeNotFound)?;
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
const harness = require('./harness.js');
const generated = require('./elm.js');
const expectedOutput = require('{}');

harness(generated, expectedOutput);
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

    let mut runner_child = Command::new(node_exe)
        .arg("--unhandled-rejections=strict")
        .arg(&main_file)
        .stdout(Stdio::piped())
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::NodeProcess)?;

    let maybe_timedout =
        future::select(time::delay_for(config.run_timeout()), &mut runner_child).await;

    let runner_status = match maybe_timedout {
        Either::Left(((), child)) => {
            child.kill().map_err(RunError::NodeProcess)?;
            let stdout = read_to_buf(child.stdout.as_mut().unwrap())
                .await
                .map_err(RunError::NodeProcess)?;
            let stderr = read_to_buf(child.stderr.as_mut().unwrap())
                .await
                .map_err(RunError::NodeProcess)?;
            return Err(RunError::Timeout {
                after: config.run_timeout(),
                stdout,
                stderr,
            });
        }
        Either::Right((status, _)) => status.map_err(RunError::NodeProcess)?,
    };

    let stdout = read_to_buf(runner_child.stdout.unwrap())
        .await
        .map_err(RunError::NodeProcess)?;
    let stderr = read_to_buf(runner_child.stderr.unwrap())
        .await
        .map_err(RunError::NodeProcess)?;

    let output = Output {
        status: runner_status,
        stdout,
        stderr,
    };

    if !output.status.success() {
        return Err(RunError::Runtime(output));
    }
    if !output.stdout.is_empty() {
        return Err(RunError::OutputProduced(output));
    }
    let possible_stderr = |mode| {
        format!(
            "Compiled in {} mode. Follow the advice at https://elm-lang.org/0.19.1/optimize for better performance and smaller assets.\n",
            mode
        ).into_bytes()
    };
    if !output.stderr.is_empty()
        && output.stderr[..] != *possible_stderr("DEV")
        && output.stderr[..] != *possible_stderr("DEBUG")
    {
        return Err(RunError::OutputProduced(output));
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

pub async fn compile_and_run<Ps: AsRef<Path>, Pe: AsRef<Path>>(
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
    let failure_allowed = instructions.config.allowed_failures().iter().any(|p| {
        if p.as_ref().exists() {
            same_file::is_same_file(&suite, p).unwrap_or_else(|e| {
                panic!(
                    "Error when comparing the paths {:?} and {:?}: {:?}",
                    suite.as_ref(),
                    p.as_ref(),
                    e
                )
            })
        } else {
            false
        }
    });

    let mut out_dir = if let Some(dir) = provided_out_dir {
        OutDir::Provided(dir)
    } else {
        let dir = tempfile::Builder::new()
            .prefix("elm-torture")
            .tempdir()
            .expect("Should be able to create a temp_file");
        OutDir::Tempory(dir)
    };

    super::suite::compile(suite.as_ref(), out_dir.path(), &instructions.config)
        .await
        .map_err(|e| CompileAndRunError::CompileFailure {
            allowed: failure_allowed,
            reason: e,
        })?;

    super::suite::run(suite.as_ref(), out_dir.path(), &instructions.config)
        .await
        .map_err(|e| {
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
/**/
pub fn compile_and_run_suites<'a, Ps: AsRef<Path> + 'a>(
    suites: impl Stream<Item = Ps> + 'a,
    instructions: &'a super::cli::Instructions,
) -> impl Stream<Item = (Ps, Result<(), CompileAndRunError<&'static Path>>)> + 'a {
    async fn scanner<'a, P: AsRef<Path>>(
        prev_run_failed: Arc<AtomicBool>,
        instructions: &'a super::cli::Instructions,
        suite: P,
    ) -> Option<(P, Result<(), CompileAndRunError<&'static Path>>)> {
        if instructions.fail_fast && prev_run_failed.load(Ordering::Relaxed) {
            None
        } else {
            let res: Result<(), CompileAndRunError<&Path>> =
                compile_and_run(&suite, None, instructions).await;
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
            // Never clear `prev_run_failed`, only set it.
            prev_run_failed.fetch_or(failed, Ordering::Relaxed);
            Some((suite, res))
        }
    }
    suites.scan(
        Arc::new(AtomicBool::new(false)),
        move |prev_runs_failed, suite| scanner(prev_runs_failed.clone(), instructions, suite),
    )
}
