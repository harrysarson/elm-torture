#![warn(clippy::all, clippy::pedantic)]

mod lib;

use clap::App;
use clap::Arg;
use lib::compile;
use lib::config;
use lib::config::Config;
use std::fmt;
use std::fs::File;
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
        .get_matches();

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

fn run_suite(suite: &Path, provided_out_dir: Option<&Path>, config: &Config) -> Option<NonZeroI32> {
    if !suite.exists() {
        eprintln!(
            "The provided path to a suite: \"{}\"  does not exist",
            suite.display()
        );
        return NonZeroI32::new(3);
    }
    if !suite.is_dir() {
        eprintln!(
            "The provided path to a suite: \"{}\" exists but is not a directory",
            suite.display()
        );
        return NonZeroI32::new(3);
    }
    if !suite.join("elm.json").exists() {
        eprintln!(
            "The suite directory: \"{}\" is not an Elm application or package",
            suite.display()
        );
        return NonZeroI32::new(3);
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
    println!("Testing suite: {:?}...", suite);

    let (out_dir_temp, out_dir_ref) = if let Some(dir) = provided_out_dir {
        (None, dir.to_path_buf())
    } else {
        let dir = tempfile::Builder::new()
            .prefix("elm-torture")
            .tempdir()
            .expect("Should be able to create a temp_file");
        let path = dir.path().to_path_buf();
        (Some(dir), path)
    };
    let exit_code = match lib::run_suite(&suite, &out_dir_ref, &config) {
        Err(err) => match err {
            lib::Error::Compiler(err) => {
                use compile::Error::*;
                match err {
                        CompilerNotFound(err) => {
                            eprintln!("Could not find elm compiler executable! Details:\n{}", err);
                        }
                        ReadingTargets(err) => {
                            eprintln!("targets.txt found in suite {} but could not be read!. Details:\n{}", &suite.display(), err);
                        }
                        Process(err) => {
                            panic!("Failed to execute compiler! Details:\n{}", err);
                        }
                        Compiler(output) | CompilerStdErrNotEmpty(output) => {
                            eprintln!("Compilation failed!\n{}", OutputPrinter(&output));
                        }
                        SuiteDoesNotExist => {
                            panic!("Path was not suite - this should have been checked already!");
                        }

                        OutDirIsNotDir => {
                            match provided_out_dir {
                                Some(dir) =>
                                    eprintln!("{} must either be a directory or a path where elm-torture can create one!",
                                    dir.display()),
                                None =>
                                    panic!("Invalid tempory directory: {}", out_dir_ref.display()),
                            }
                        }
                    }
                NonZeroI32::new(1)
            }
            lib::Error::Runner(err) => {
                use lib::run::Error::*;
                eprintln!("The suite {} failed at run time.", suite.display());
                match err {
                    NodeNotFound(err) => {
                        eprintln!("Could not find node executable to run generated Javascript. Details:\n{}", err);
                    }
                    SuiteDoesNotExist => {
                        panic!("Path was not suite - this should have been checked already!");
                    }
                    NodeProcess(err) => {
                        panic!("The node process errored unexpectedly:\n{}", err);
                    }
                    WritingHarness(err) => {
                        panic!(
                            "Cannot add the test harness to the output directory. Details:\n{}",
                            err
                        );
                    }
                    ExpectedOutputNotUtf8(_) => {
                        panic!("Expected output is not valid utf8");
                    }
                    CopyingExpectedOutput(err) => {
                        panic!(
                            "The expected output exists but cannot be copied. Details:\n{}",
                            err
                        );
                    }
                    Runtime(output) => {
                        eprintln!("{}", OutputPrinter(&output));
                        eprintln!(
                            "\n\nTo inspect the built files that caused this error see:\n  {}",
                            out_dir_ref.display()
                        );
                        if let Some(tempdir) = out_dir_temp {
                            tempdir.into_path();
                        }
                    }
                    CannotFindExpectedOutput => {
                        eprintln!(
                            "{}\n{}",
                            "Each suite must contain a file 'output.txt', containing the text that",
                            "the suite should write to stdout"
                        );
                    }
                    OutputProduced(output) => {
                        eprintln!(
                            "The suite ran without error but produced the following output!:\n{}",
                            OutputPrinter(&output)
                        );
                    }
                }
                NonZeroI32::new(2)
            }
        },
        Ok(()) => None,
    };
    if failure_allowed {
        if exit_code.is_some() {
            eprintln!("Failure allowed, continuing...");
            None
        } else {
            eprintln!("Suite was expected to fail but did not!");
            NonZeroI32::new(3)
        }
    } else {
        exit_code
    }
}

fn iterate_till_some<U, It, F>(iter: It, func: F) -> Option<U>
where
    It: Iterator,
    F: Fn(It::Item) -> Option<U>,
{
    for item in iter {
        if let Some(val) = func(item) {
            return Some(val);
        }
    }
    None
}

fn run_app(instructions: &CliInstructions) -> Option<NonZeroI32> {
    let CliInstructions { config, task } = instructions;
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
            run_suite(suite, out_dir.as_ref().map(PathBuf::as_ref), &config)
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
            iterate_till_some(suites.iter(), |suite| run_suite(suite, None, &config))
        }
    }
}

fn main() {
    env_logger::init();
    process::exit(run_app(&get_cli_task()).map_or(0, NonZeroI32::get));
}
