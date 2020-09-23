use super::config;
use apply::{Also, Apply};
use config::OptimizationLevel;
use core::fmt;
use io::{Read, Write};
use log::debug;
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::process::{Output, Stdio};
use std::{collections::HashMap, fs::File};
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

fn iter_pairs<T: Clone + Sync + Send, U: Send>(
    into1: impl IntoParallelIterator<Item = T, Iter = impl ParallelIterator<Item = T>>,
    into2: impl IntoParallelIterator<Item = U, Iter = impl ParallelIterator<Item = U> + Clone + Sync>,
) -> impl IntoParallelIterator<Item = (T, U)> {
    let it2 = into2.into_par_iter();
    into1
        .into_par_iter()
        .flat_map(move |v1| it2.clone().map(move |v2| (v1.clone(), v2)))
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ElmCompilerPath {
    unresolved: String,
    path: PathBuf,
    pub stdlib_variant: StdlibVariant,
}

impl fmt::Display for ElmCompilerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.unresolved)
    }
}

impl ElmCompilerPath {
    fn new_resolved(binary_name: String) -> Result<Self, DetectStdlibError> {
        use bstr::ByteSlice;
        let path = which::which(&binary_name).map_err(DetectStdlibError::LocatingCompiler)?;
        let mut command = Command::new(&path);
        command.arg("--stdlib-variant");
        set_elm_home(&mut command);

        debug!("Invoking compiler to detect stdlib variant: {:?}", command);

        let Output { status, stdout, .. } = command.output().map_err(DetectStdlibError::Io)?;

        let stdlib_variant = if status.success() {
            if stdout.trim().starts_with(b"another-elm") {
                Ok(StdlibVariant::Another)
            } else {
                Err(DetectStdlibError::Parsing(stdout.into_boxed_slice()))
            }
        } else {
            Ok(StdlibVariant::Official)
        }?;
        Ok(Self {
            unresolved: binary_name,
            path,
            stdlib_variant,
        })
    }

    fn command(&self) -> Command {
        Command::new(&self.path)
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
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum StdlibVariant {
    Official,
    Another,
}
#[serde(rename_all = "lowercase")]
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Platform {
    Linux,
    MacOs,
    Windows,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct RunFailsIfAll {
    stdlib_variant: AnyOneOf<StdlibVariant>,
    opt_level: AnyOneOf<config::OptimizationLevel>,
    platform: AnyOneOf<Platform>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CompileFailsIfAll {
    opt_level: AnyOneOf<config::OptimizationLevel>,
    platform: AnyOneOf<Platform>,
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
    opt_level: config::OptimizationLevel,
    stdlib_variant: StdlibVariant,
    platform: Platform,
}

impl Condition for RunFailsIfAll {
    type Facts = RunFailsIfAllFacts;
    fn is_met(&self, f: &Self::Facts) -> bool {
        self.opt_level.any(|level| *level == f.opt_level)
            && self
                .stdlib_variant
                .any(|variant| *variant == f.stdlib_variant)
            && self.platform.any(|platform| *platform == f.platform)
    }
}
struct CompileFailsIfAllFacts {
    opt_level: config::OptimizationLevel,
    platform: Platform,
}

impl Condition for CompileFailsIfAll {
    type Facts = CompileFailsIfAllFacts;
    fn is_met(&self, f: &Self::Facts) -> bool {
        self.opt_level.any(|level| *level == f.opt_level)
            && self.platform.any(|platform| *platform == f.platform)
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
    Process(io::Error),
    Compiler(Output),
    CompilerStdErrNotEmpty(Output),
    ReadingTargets(io::Error),
    DeletingElmStuff(io::Error),
    SuiteDoesNotExist,
}

#[derive(Debug)]
pub enum DetectStdlibError {
    Io(io::Error),
    LocatingCompiler(which::Error),
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
pub enum CompileAndRunError {
    SuiteNotExist,
    SuiteNotDir,
    SuiteNotElm,
    OutDirIsNotDir,
    CannotGetSuiteConfig(GetSuiteConfigError),
    CompileFailure {
        allowed: bool,
        reason: super::suite::CompileError,
    },
    RunFailure {
        allowed: bool,
        reason: super::suite::RunError,
    },
    ExpectedCompileFailure,
    ExpectedRunFailure,
}

fn set_elm_home(command: &mut Command) {
    if let Some(elm_home) = env::var_os("ELM_HOME") {
        command.env("ELM_HOME", elm_home);
    }
}

fn compile(
    suite: &Path,
    out_file: impl AsRef<Path>,
    compiler_lock: &Mutex<()>,
    opt_level: OptimizationLevel,
    compiler_path: &ElmCompilerPath,
    config: &config::Config,
) -> Result<(), CompileError> {
    fn compile_help(
        suite: impl AsRef<Path>,
        command: &mut Command,
    ) -> Result<Output, CompileError> {
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
    let mut command = compiler_path.command();

    command.current_dir(suite);
    command.arg("make");
    command.args(root_files);
    command.args(opt_level.args().iter());
    command.arg("--output");
    set_elm_home(&mut command);
    command.arg(out_file.as_ref());

    debug!("Invoking compiler: {:?}", command);

    let res = run_until_success(config.compiler_max_retries(), || {
        let _lock = compiler_lock.lock();
        compile_help(&suite, &mut command)
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
fn run(
    suite: &Path,
    out_dir: &Path,
    opt_level: OptimizationLevel,
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
        .apply(|mut f| {
            write!(
                f,
                r#"
const harness = require('./harness.js');
const generated = require('./elm-{}.js');
const expectedOutput = require('./output.json');

harness(generated, expectedOutput);
"#,
                opt_level.id()
            )
        })
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

pub type SscceRunType = (ElmCompilerPath, OptimizationLevel);

fn compile_and_run(
    suite: impl AsRef<Path> + Sync,
    out_dir: impl AsRef<Path> + Sync,
    compiler_lock: &Mutex<()>,
    configurations: impl IntoParallelIterator<Item = SscceRunType>,
    config: &config::Config,
) -> HashMap<SscceRunType, Result<(), CompileAndRunError>> {
    let platform = match env::consts::OS {
        "linux" => Platform::Linux,
        "macos" => Platform::MacOs,
        "windows" => Platform::Windows,
        _ => panic!("Unsupported platform. (Add it to Platform enum!)"),
    };
    configurations
        .into_par_iter()
        .map(|(elm_compiler, opt_level)| {
            let res = (|| {
                if !suite.as_ref().exists() {
                    return Err(CompileAndRunError::SuiteNotExist);
                }
                if !suite.as_ref().is_dir() {
                    return Err(CompileAndRunError::SuiteNotDir);
                }
                if !suite.as_ref().join("elm.json").exists() {
                    return Err(CompileAndRunError::SuiteNotElm);
                }

                let suite_config =
                    get_suite_config(&suite).map_err(CompileAndRunError::CannotGetSuiteConfig)?;

                let compile_failure_allowed =
                    suite_config
                        .compile_fails_if
                        .is_met(&CompileFailsIfAllFacts {
                            opt_level,
                            platform,
                        });

                compile(
                    suite.as_ref(),
                    out_dir.as_ref().join(format!("elm-{}.js", opt_level.id())),
                    &compiler_lock,
                    opt_level,
                    &elm_compiler,
                    &config,
                )
                .map_err(|e| CompileAndRunError::CompileFailure {
                    allowed: compile_failure_allowed,
                    reason: e,
                })?;

                if compile_failure_allowed {
                    return Err(CompileAndRunError::ExpectedCompileFailure);
                }

                let run_failure_allowed = suite_config.run_fails_if.is_met(&RunFailsIfAllFacts {
                    opt_level,
                    stdlib_variant: elm_compiler.stdlib_variant,
                    platform,
                });

                debug!(
                    "Runtime failure allowed? {:?}. Config: {:?}. Actual {:?}",
                    run_failure_allowed, &suite_config.run_fails_if, elm_compiler.stdlib_variant
                );

                run(
                    suite.as_ref(),
                    out_dir.as_ref(),
                    opt_level,
                    &config,
                    &suite_config,
                )
                .map_err(|e| CompileAndRunError::RunFailure {
                    allowed: run_failure_allowed,
                    reason: e,
                })?;

                if run_failure_allowed {
                    return Err(CompileAndRunError::ExpectedRunFailure);
                }

                Ok(())
            })();
            ((elm_compiler, opt_level), res)
        })
        .collect()
}

pub struct CompileAndRunResults<Ps> {
    pub suite: Ps,
    // TODO(harry): move into RunError!
    pub sscce_out_dir: PathBuf,
    /// None indicates that elm-torture ran SSCCE successfully.
    pub errors: HashMap<SscceRunType, Option<CompileAndRunError>>,
}

pub enum SuitesError {
    ResolvingCompiler(DetectStdlibError),
    // CompilerNotFound(which::Error),
    // CannotDetectStdlibVariant(DetectStdlibError),
}

#[allow(clippy::too_many_lines)]
pub fn compile_and_run_suites<'a, Ps: AsRef<Path> + Send + Sync + 'a>(
    suites: impl IntoParallelIterator<Item = Ps> + 'a,
    instructions: &'a super::cli::Instructions,
) -> Result<impl IntoParallelIterator<Item = CompileAndRunResults<Ps>> + 'a, SuitesError> {
    let (tmp_dir_raw, out_dir) = if let Some(out_dir) = &instructions.config.out_dir {
        (None, out_dir.to_path_buf())
    } else {
        let td = tempfile::Builder::new()
            .prefix("elm-torture")
            .tempdir()
            .expect("Should be able to create a temp_file");
        let pb = td.path().to_path_buf();
        (Some(td), pb)
    };
    let tmp_dir = Mutex::new(tmp_dir_raw);

    if !out_dir.exists() {
        let _ = fs::create_dir(&out_dir);
    }
    let compiler_lock = Mutex::new(());
    let prev_runs_failed = AtomicBool::new(false);

    let elm_compilers = instructions
        .config
        .elm_compilers()
        .iter()
        .map(|s| ElmCompilerPath::new_resolved(s.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(SuitesError::ResolvingCompiler)?;

    let scanner = move |suite: Ps| {
        if instructions.fail_fast && prev_runs_failed.load(Ordering::Relaxed) {
            None
        } else {
            let sscce_out_dir = out_dir.join(suite.as_ref().file_name().expect("todo"));

            if !sscce_out_dir.exists() {
                let _ = fs::create_dir(&sscce_out_dir);
            }
            if !sscce_out_dir.is_dir() {
                // TODO(harry): handle this error better
                return Some(CompileAndRunResults {
                    suite,
                    sscce_out_dir,
                    errors: HashMap::new().also(|hm| {
                        hm.insert(
                            (
                                elm_compilers[0].clone(),
                                instructions.config.opt_levels()[0],
                            ),
                            Some(CompileAndRunError::OutDirIsNotDir),
                        );
                    }),
                });
            }

            let errors = compile_and_run(
                &suite,
                &sscce_out_dir,
                &compiler_lock,
                iter_pairs(
                    elm_compilers.clone(),
                    instructions.config.opt_levels().par_iter().copied(),
                ),
                &instructions.config,
            )
            .into_iter()
            .map(|(opt_level, res)| {
                if let Err(CompileAndRunError::RunFailure { .. }) = res {
                    if let Some(dir) = tmp_dir.lock().unwrap().take() {
                        dir.into_path();
                    }
                } else {
                    let _ = fs::remove_dir_all(&sscce_out_dir);
                };
                let failed = match res {
                    Err(CompileAndRunError::CompileFailure { allowed: true, .. })
                    | Err(CompileAndRunError::RunFailure { allowed: true, .. })
                    | Ok(_) => false,
                    Err(_) => true,
                };
                // Never clear `prev_run_failed`, only set it.
                prev_runs_failed.fetch_or(failed, Ordering::Relaxed);
                (opt_level, res.err())
            })
            .collect::<HashMap<_, _>>();
            Some(CompileAndRunResults {
                suite,
                sscce_out_dir,
                errors,
            })
        }
    };
    suites.into_par_iter().map(scanner).while_some().apply(Ok)
}
