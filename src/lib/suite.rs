use super::config;
use io::{Read, Write};
use log::debug;
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::process::{Output, Stdio};
use std::{env, ffi::OsStr};
use std::{fs, sync::Mutex};
use std::{io, process::Command};
use std::{
    path::Path,
    path::PathBuf,
    string,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use wait_timeout::ChildExt;

type AnyOneOf<T> = Option<Box<[T]>>;

trait AnyOneOfExt {
    type Item;
    fn any(&self, f: impl FnMut(&Self::Item) -> bool) -> bool;
}

impl<T> AnyOneOfExt for AnyOneOf<T> {
    type Item = T;
    fn any(&self, f: impl FnMut(&Self::Item) -> bool) -> bool {
        self.as_ref().map_or_else(|| true, |s| s.iter().any(f))
    }
}

fn run_until_success<T, E>(
    max_retries: usize,
    mut f: impl FnMut() -> Result<T, E>,
) -> Result<T, E> {
    for _ in 0..max_retries {
        if let Ok(val) = f() {
            return Ok(val);
        }
    }
    f()
}

trait ResultFlip {
    type Err;
    type Ok;
    fn flip(self) -> Result<Self::Err, Self::Ok>;
}

impl<T, E> ResultFlip for Result<T, E> {
    type Err = E;
    type Ok = T;
    fn flip(self) -> Result<E, T> {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortType {
    Command,
    Subscription,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PortName(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct PortArg(serde_json::Value);

#[serde(rename_all = "kebab-case")]
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Copy)]
pub enum StdlibVariant {
    Official,
    Another,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct RunFailsIfAll {
    stdlib_variant: AnyOneOf<StdlibVariant>,
    opt_level: AnyOneOf<config::OptimisationLevel>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CompileFailsIfAll {
    opt_level: AnyOneOf<config::OptimisationLevel>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged, rename_all = "kebab-case")]
pub enum ConditionCollection<C> {
    Collection(ConditionCollectionHelper<C>),
    Cond(C),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConditionCollectionHelper<C> {
    All(Box<[ConditionCollection<C>]>),
    Any(Box<[ConditionCollection<C>]>),
}

trait Condition {
    type Facts;
    fn is_met(&self, f: &Self::Facts) -> bool;
}

impl<C: Condition> Condition for ConditionCollection<C> {
    type Facts = C::Facts;
    fn is_met(&self, f: &Self::Facts) -> bool {
        match self {
            Self::Collection(ConditionCollectionHelper::All(cs)) => cs.iter().all(|c| c.is_met(f)),
            Self::Collection(ConditionCollectionHelper::Any(cs)) => cs.iter().any(|c| c.is_met(f)),
            Self::Cond(c) => c.is_met(f),
        }
    }
}

impl<C: Condition> Condition for Option<C> {
    type Facts = C::Facts;
    fn is_met(&self, f: &Self::Facts) -> bool {
        self.as_ref().map_or(false, |c| c.is_met(f))
    }
}

struct RunFailsIfAllFacts {
    opt_level: config::OptimisationLevel,
    stdlib_variant: StdlibVariant,
}

impl Condition for RunFailsIfAll {
    type Facts = RunFailsIfAllFacts;
    fn is_met(&self, f: &Self::Facts) -> bool {
        self.opt_level.any(|level| *level == f.opt_level)
            && self
                .stdlib_variant
                .any(|variant| *variant == f.stdlib_variant)
    }
}
struct CompileFailsIfAllFacts {
    opt_level: config::OptimisationLevel,
}

impl Condition for CompileFailsIfAll {
    type Facts = CompileFailsIfAllFacts;
    fn is_met(&self, f: &Self::Facts) -> bool {
        self.opt_level.any(|level| *level == f.opt_level)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    ports: Option<Box<[(PortType, PortName, PortArg)]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compile_fails_if: Option<ConditionCollection<CompileFailsIfAll>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run_fails_if: Option<ConditionCollection<RunFailsIfAll>>,
}

#[derive(Debug)]
pub enum CompileError {
    CompilerNotFound(which::Error),
    Process(io::Error),
    Compiler(Output),
    CompilerStdErrNotEmpty(Output),
    ReadingTargets(io::Error),
    DeletingElmStuff(io::Error),
    SuiteDoesNotExist,
    OutDirIsNotDir,
}

#[derive(Debug)]
pub enum DetectStdlibError {
    Io(io::Error),
    Parsing(Box<[u8]>),
}
#[derive(Debug)]
pub enum GetSuiteConfigError {
    CannotRead(io::Error),
    Parse(serde_json::Error),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RunError {
    NodeNotFound(which::Error),
    SuiteDoesNotExist,
    NodeProcess(io::Error),
    WritingHarness(io::Error),
    CopyingExpectedOutput(io::Error),
    Runtime(Output),
    WritingExpectedOutput(io::Error),
    ExpectedOutputNotUtf8(string::FromUtf8Error),
    OutputProduced(Output),
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
    CannotGetSuiteConfig(GetSuiteConfigError),
    CompileFailure {
        allowed: bool,
        reason: super::suite::CompileError,
    },
    RunFailure {
        allowed: bool,
        outdir: OutDir<P>,
        reason: super::suite::RunError,
    },
    ExpectedCompileFailure,
    ExpectedRunFailure,
    CannotDetectStdlibVariant(DetectStdlibError),
}

#[derive(Debug)]
pub enum OutDir<P> {
    Provided(P),
    Tempory(tempfile::TempDir),
    Persistent(PathBuf),
}

pub fn compile(
    suite: &Path,
    out_dir: &Path,
    compiler_lock: &Mutex<()>,
    config: &config::Config,
) -> Result<(), CompileError> {
    fn compile(suite: impl AsRef<Path>, command: &mut Command) -> Result<Output, CompileError> {
        fs::remove_dir_all(suite.as_ref().join("elm-stuff"))
            .or_else(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .map_err(CompileError::DeletingElmStuff)?;
        command.output().map_err(CompileError::Process)
    };

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

    let res = run_until_success(config.compiler_max_retries(), || {
        let _lock = compiler_lock.lock();
        compile(&suite, &mut command)
    })?;

    if !res.status.success() {
        return Err(CompileError::Compiler(res));
    }

    if !res.stderr.is_empty() {
        return Err(CompileError::CompilerStdErrNotEmpty(res));
    }

    Ok(())
}

fn get_suite_config(suite: impl AsRef<Path>) -> Result<Config, GetSuiteConfigError> {
    let expected_output_path = suite.as_ref().join("output.json");
    serde_json::from_slice(
        &fs::read(expected_output_path).map_err(GetSuiteConfigError::CannotRead)?,
    )
    .map_err(GetSuiteConfigError::Parse)
}

#[allow(clippy::too_many_lines)]
pub fn run(
    suite: &Path,
    out_dir: &Path,
    config: &config::Config,
    suite_config: &Config,
) -> Result<(), RunError> {
    fn read_to_buf(mut read: impl io::Read) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // read the whole file
        read.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    if !suite.join("elm.json").exists() {
        return Err(RunError::SuiteDoesNotExist);
    }
    let node_exe = which::which(&config.node()).map_err(RunError::NodeNotFound)?;
    let harness_file = out_dir.join("harness.js");
    let output_file = out_dir.join("output.json");
    let main_file = out_dir.join("main.js");

    fs::write(
        &harness_file,
        &include_bytes!("../../embed-assets/run.js")[..],
    )
    .map_err(RunError::WritingHarness)?;
    fs::write(
        &output_file,
        &serde_json::to_vec_pretty(&suite_config).expect("Failed to reserialize output json"),
    )
    .map_err(RunError::WritingExpectedOutput)?;

    File::create(&main_file)
        .map_err(RunError::WritingHarness)?
        .write_all(
            br#"
const harness = require('./harness.js');
const generated = require('./elm.js');
const expectedOutput = require('./output.json');

harness(generated, expectedOutput);
"#,
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

    let runner_status = runner_child
        .wait_timeout(config.run_timeout())
        .map_err(RunError::NodeProcess)?
        .map_or_else(
            || {
                runner_child.kill().map_err(RunError::NodeProcess)?;
                let stdout = read_to_buf(runner_child.stdout.as_mut().unwrap())
                    .map_err(RunError::NodeProcess)?;
                let stderr = read_to_buf(runner_child.stderr.as_mut().unwrap())
                    .map_err(RunError::NodeProcess)?;
                Err(RunError::Timeout {
                    after: config.run_timeout(),
                    stdout,
                    stderr,
                })
            },
            Ok,
        )?;

    let stdout = read_to_buf(runner_child.stdout.unwrap()).map_err(RunError::NodeProcess)?;
    let stderr = read_to_buf(runner_child.stderr.unwrap()).map_err(RunError::NodeProcess)?;

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

fn detect_stdlib_variant(
    elm_compiler: impl AsRef<OsStr>,
) -> Result<StdlibVariant, DetectStdlibError> {
    use bstr::ByteSlice;
    let mut command = Command::new(elm_compiler);
    command.arg("--stdlib-variant");

    debug!("Invoking compiler to detect stdlib variant: {:?}", command);

    let Output { status, stdout, .. } = command.output().map_err(DetectStdlibError::Io)?;

    if status.success() {
        if stdout.trim().starts_with(b"another-elm") {
            Ok(StdlibVariant::Another)
        } else {
            Err(DetectStdlibError::Parsing(stdout.into_boxed_slice()))
        }
    } else {
        Ok(StdlibVariant::Official)
    }
}

pub fn compile_and_run<Ps: AsRef<Path>, Pe: AsRef<Path>>(
    suite: Ps,
    provided_out_dir: Option<Pe>,
    compiler_lock: &Mutex<()>,
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

    let mut out_dir = provided_out_dir.map_or_else(
        || {
            let dir = tempfile::Builder::new()
                .prefix("elm-torture")
                .tempdir()
                .expect("Should be able to create a temp_file");
            OutDir::Tempory(dir)
        },
        OutDir::Provided,
    );

    let suite_config =
        get_suite_config(&suite).map_err(CompileAndRunError::CannotGetSuiteConfig)?;

    let compile_failure_allowed = suite_config
        .compile_fails_if
        .is_met(&CompileFailsIfAllFacts {
            opt_level: instructions.config.opt_level(),
        });

    compile(
        suite.as_ref(),
        out_dir.path(),
        &compiler_lock,
        &instructions.config,
    )
    .map_err(|e| CompileAndRunError::CompileFailure {
        allowed: compile_failure_allowed,
        reason: e,
    })?;

    if compile_failure_allowed {
        return Err(CompileAndRunError::ExpectedCompileFailure);
    }

    let actual_stdlib_variant = detect_stdlib_variant(instructions.config.elm_compiler())
        .map_err(CompileAndRunError::CannotDetectStdlibVariant)?;

    let run_failure_allowed = suite_config.run_fails_if.is_met(&RunFailsIfAllFacts {
        opt_level: instructions.config.opt_level(),
        stdlib_variant: actual_stdlib_variant,
    });

    debug!(
        "Runtime failure allowed? {:?}. Config: {:?}. Actual {:?}",
        run_failure_allowed, &suite_config.run_fails_if, &actual_stdlib_variant
    );

    run(
        suite.as_ref(),
        out_dir.path(),
        &instructions.config,
        &suite_config,
    )
    .map_err(|e| {
        out_dir.persist();
        CompileAndRunError::RunFailure {
            allowed: run_failure_allowed,
            outdir: out_dir,
            reason: e,
        }
    })?;

    if run_failure_allowed {
        return Err(CompileAndRunError::ExpectedRunFailure);
    }

    Ok(())
}
/**/
pub fn compile_and_run_suites<'a, Ps: AsRef<Path> + Send + 'a>(
    suites: impl IntoParallelIterator<Item = Ps> + 'a,
    instructions: &'a super::cli::Instructions,
) -> impl IntoParallelIterator<Item = (Ps, Result<(), CompileAndRunError<&'static Path>>)> + 'a {
    fn scanner<'a, P: AsRef<Path>>(
        prev_run_failed: &AtomicBool,
        compiler_lock: &Mutex<()>,
        instructions: &'a super::cli::Instructions,
        suite: P,
    ) -> Option<(P, Result<(), CompileAndRunError<&'static Path>>)> {
        if instructions.fail_fast && prev_run_failed.load(Ordering::Relaxed) {
            None
        } else {
            let res: Result<(), CompileAndRunError<&Path>> =
                compile_and_run(&suite, None, compiler_lock, instructions);
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
    let compiler_lock = Mutex::new(());
    let prev_runs_failed = AtomicBool::new(false);
    suites
        .into_par_iter()
        .map(move |suite| scanner(&prev_runs_failed, &compiler_lock, instructions, suite))
        .while_some()
}
