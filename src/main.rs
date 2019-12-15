#![warn(clippy::all, clippy::pedantic)]

mod lib;

use clap::App;
use clap::Arg;
use lib::compile;
use lib::config;
use lib::config::Config;
use lib::run;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::mem;
use std::num::NonZeroI32;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Output;

enum CliTask {
    DumpConfig,
    RunSuite {
        suite: PathBuf,
        out_dir: Option<PathBuf>,
    },
    RunSuites(PathBuf),
}

struct CliInstructions {
    config: config::Config,
    clear_elm_stuff: bool,
    task: CliTask,
}

fn get_cli_task() -> CliInstructions {
    let matches = App::new("Elm Torture")
        .version("0.0.1")
        .author("Harry Sarson <harry.sarson@hotmail.co.uk>")
        .about("Test suite for an elm compiler")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Set config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("suite")
                .long("suite")
                .value_name("DIRECTORY")
                .help("The suite to test")
                .required(true)
                .conflicts_with("suites")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("suites")
                .long("suites")
                .value_name("DIRECTORY")
                .help("A directory containing suites to test")
                .conflicts_with("out_dir")
                .takes_value (true))
        .arg(
            Arg::with_name("out_dir")
                .long("out-dir")
                .value_name("DIRECTORY")
                .help("The directory to place built files in.\nMust not exist or be an empty directory")
                .takes_value(true)  )
        .arg(
            Arg::with_name("show_config")
                .long("showConfig")
                .help("Dump the configuration"),
        )
        .arg(
            Arg::with_name("clear_elm_stuff")
                .long("clear-elm-stuff")
                .help("Delete the elm-stuff directory before running suite"),
        )
        .get_matches();

    let clear_elm_stuff = matches.is_present("clear_elm_stuff");

    let config = {
        let config_file = matches.value_of_os("config");

        let mut deserialised: Config = config_file
            .map(|p| File::open(p).expect("config file not found"))
            .map(|file| {
                serde_json::from_reader(file).expect("error while reading json configuration file")
            })
            .unwrap_or_default();

        if let Some(config_dir) = config_file.map(Path::new).and_then(Path::parent) {
            deserialised.allowed_failures = deserialised
                .allowed_failures
                .iter()
                .map(|p| config_dir.join(p))
                .collect();
        }
        deserialised
    };

    CliInstructions {
        config,
        clear_elm_stuff,
        task: if matches.is_present("show_config") {
            CliTask::DumpConfig
        } else if let Some(suites) = matches.value_of("suites") {
            CliTask::RunSuites(
                suites
                    .parse()
                    .unwrap_or_else(|infalible| match infalible {}),
            )
        } else {
            CliTask::RunSuite {
                suite: matches
                    .value_of("suite")
                    .unwrap()
                    .parse()
                    .unwrap_or_else(|infalible| match infalible {}),
                out_dir: matches
                    .value_of("out_dir")
                    .map(|dir| dir.parse().unwrap_or_else(|infalible| match infalible {})),
            }
        },
    }
}

struct OutputPrinter<'a>(&'a Output);

impl<'a> fmt::Display for OutputPrinter<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let OutputPrinter(output) = self;
        write!(
            fmt,
            r#"
 = Exit code: {} =
 = Std Out =
{}
 = Std Err =
{}"#,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

#[derive(Debug)]
enum OutDir<'a> {
    Provided(&'a Path),
    Tempory(tempfile::TempDir),
    Persistent(PathBuf),
}

impl<'a> OutDir<'a> {
    fn path(&self) -> &Path {
        match self {
            Self::Provided(p) => p,
            Self::Tempory(ref p) => p.path(),
            Self::Persistent(ref p) => p,
        }
    }

    fn is_tempory(&self) -> bool {
        match self {
            Self::Provided(_) => false,
            Self::Tempory(_) | Self::Persistent(_) => true,
        }
    }

    fn persist(&mut self) {
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

struct CompilerError<'a> {
    err: &'a compile::Error,
    suite: &'a Path,
    out_dir: &'a OutDir<'a>,
}

impl<'a> fmt::Display for CompilerError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use compile::Error::*;
        match self.err {
            CompilerNotFound(err) => write!(
                f,
                "Could not find elm compiler executable! Details:\n{}",
                err
            ),
            ReadingTargets(err) => write!(
                f,
                "targets.txt found in suite {} but could not be read!. Details:\n{}",
                self.suite.display(),
                err
            ),
            Process(err) => panic!("Failed to execute compiler! Details:\n{}", err),
            Compiler(output) | CompilerStdErrNotEmpty(output) => {
                write!(f, "Compilation failed!\n{}", OutputPrinter(&output))
            }
            SuiteDoesNotExist => {
                panic!("Path was not suite - this should have been checked already!")
            }

            OutDirIsNotDir => {
                if self.out_dir.is_tempory() {
                    panic!(
                        "Invalid tempory directory: {}",
                        self.out_dir.path().display()
                    )
                } else {
                    write!(
                        f,
                        "{} must either be a directory or a path where elm-torture can create one!",
                        self.out_dir.path().display()
                    )
                }
            }
        }
    }
}

fn display_compiler_error<'a>(
    err: &'a compile::Error,
    suite: &'a Path,
    out_dir: &'a OutDir<'a>,
) -> CompilerError<'a> {
    CompilerError {
        err,
        suite,
        out_dir,
    }
}

struct RuntimeError<'a> {
    err: &'a run::Error,
    out_dir: &'a Path,
}

impl<'a> fmt::Display for RuntimeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use run::Error::*;
        match self.err {
            NodeNotFound(err) => write!(
                f,
                "Could not find node executable to run generated Javascript. Details:\n{}",
                err
            ),
            SuiteDoesNotExist => {
                panic!("Path was not suite - this should have been checked already!")
            }
            NodeProcess(err) => panic!("The node process errored unexpectedly:\n{}", err),
            WritingHarness(err) => panic!(
                "Cannot add the test harness to the output directory. Details:\n{}",
                err
            ),
            ExpectedOutputNotUtf8(_) => panic!("Expected output is not valid utf8"),
            CopyingExpectedOutput(err) => panic!(
                "The expected output exists but cannot be copied. Details:\n{}",
                err
            ),
            Runtime(output) => {
                write!(f, "{}", OutputPrinter(&output))?;
                write!(
                    f,
                    "\n\nTo inspect the built files that caused this error see:\n  {}",
                    self.out_dir.display()
                )
            }
            CannotFindExpectedOutput => write!(
                f,
                "{}",
                [
                    "Each suite must contain a file 'output.txt', containing the text that",
                    "the suite should write to stdout"
                ]
                .join("\n")
            ),
            OutputProduced(output) => write!(
                f,
                "The suite ran without error but produced the following output!:\n{}",
                OutputPrinter(&output)
            ),
        }
    }
}

fn display_runner_error<'a>(err: &'a run::Error, out_dir: &'a Path) -> RuntimeError<'a> {
    RuntimeError { err, out_dir }
}

struct SuiteFailure<'a> {
    suite: &'a Path,
    outdir: OutDir<'a>,
    reason: lib::Error,
}

enum SuiteError<'a> {
    SuiteNotExist(&'a Path),
    SuiteNotDir(&'a Path),
    SuiteNotElm(&'a Path),
    Failure {
        allowed: bool,
        reason: SuiteFailure<'a>,
    },
    ExpectedFailure,
}

impl<'a> fmt::Display for SuiteError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SuiteError::*;

        match self {
            SuiteNotExist(suite) => write!(
                f,
                "The provided path to a suite: \"{}\"  does not exist",
                suite.display()
            ),

            SuiteNotDir(suite) => write!(
                f,
                "The provided path to a suite: \"{}\" exists but is not a directory",
                suite.display()
            ),

            SuiteNotElm(suite) => write!(
                f,
                "The suite directory: \"{}\" is not an Elm application or package",
                suite.display()
            ),

            Failure { allowed, reason } => {
                match &reason.reason {
                    lib::Error::Compiler(err) => write!(
                        f,
                        "Failed to compile suite {}.\n{}",
                        &reason.suite.display(),
                        indented::indented(display_compiler_error(
                            &err,
                            &reason.suite,
                            &reason.outdir
                        ))
                    ),

                    lib::Error::Runner(err) => write!(
                        f,
                        "Suite {} failed at run time.\n{}",
                        &reason.suite.display(),
                        indented::indented(display_runner_error(&err, reason.outdir.path()))
                    ),
                }?;
                if *allowed {
                    write!(f, "Failure allowed, continuing...")
                } else {
                    Ok(())
                }
            }

            ExpectedFailure => write!(f, "Suite was expected to fail but did not!"),
        }
    }
}

fn run_suite<'a>(
    suite: &'a Path,
    provided_out_dir: Option<&'a Path>,
    clear_elm_stuff: bool,
    config: &Config,
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
    let failure_allowed = config.allowed_failures.iter().any(|p| {
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
    if clear_elm_stuff {
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

    let run_result = lib::run_suite(&suite, out_dir.path(), &config);

    if let Err(lib::Error::Runner(run::Error::Runtime(_))) = run_result {
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

fn get_exit_code(suite_result: &Result<(), SuiteError>) -> i32 {
    use SuiteError::*;

    match suite_result {
        Err(ref suite_error) => match suite_error {
            SuiteNotExist(_) | SuiteNotDir(_) | SuiteNotElm(_) => 0x28,

            Failure { reason, allowed } => {
                if *allowed {
                    0
                } else {
                    match reason.reason {
                        lib::Error::Compiler(_) => 0x21,
                        lib::Error::Runner(_) => 0x22,
                    }
                }
            }

            ExpectedFailure => 0x24,
        },
        Ok(()) => 0,
    }
}

fn run_app(instructions: &CliInstructions) -> Option<NonZeroI32> {
    let CliInstructions {
        config,
        clear_elm_stuff,
        task,
    } = instructions;
    let welcome_message = "Elm Torture - stress tests for an elm compiler";
    match task {
        CliTask::DumpConfig => {
            println!(
                "{}",
                serde_json::to_string_pretty(&config).expect("could not serialize config")
            );
            None
        }
        CliTask::RunSuite { suite, out_dir } => {
            println!("{}", welcome_message);
            println!();
            println!("Running SSCCE {}:", suite.display());
            println!();
            let suite_result = run_suite(
                suite,
                out_dir.as_ref().map(PathBuf::as_ref),
                *clear_elm_stuff,
                &config,
            );
            if let Err(ref e) = suite_result {
                println!("{}", e);
            }
            NonZeroI32::new(get_exit_code(&suite_result))
        }
        CliTask::RunSuites(suite_dir) => {
            let suites =
                lib::find_suites::find_suites(&suite_dir).expect("error scanning for suites");
            println!("{}", welcome_message);
            println!();
            {
                let len = suites.len();
                println!(
                    "Running the following {} SSCCE{}:",
                    len,
                    if len == 1 { "" } else { "s" }
                );
            };
            for path in suites.iter() {
                println!("  {}", path.display());
            }
            println!();

            let mut code = 0;
            for suite in suites.iter() {
                let suite_result = run_suite(suite, None, *clear_elm_stuff, &config);
                if let Err(ref e) = suite_result {
                    println!("{}", e);
                }
                code |= get_exit_code(&suite_result);
            }
            NonZeroI32::new(code)
        }
    }
}

fn main() {
    env_logger::init();
    process::exit(run_app(&get_cli_task()).map_or(0, NonZeroI32::get));
}
