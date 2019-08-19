#![warn(clippy::all, clippy::pedantic)]

mod lib;

use clap::App;
use clap::Arg;
use lib::compile;
use lib::config;
use lib::config::Config;
use std::env;
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
                .conflicts_with("show_config")
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

    let config = matches
        .value_of_os("config")
        .map(|p| File::open(p).expect("config file not found"))
        .map(|file| {
            serde_json::from_reader(file).expect("error while reading json configuration file")
        })
        .unwrap_or_default();

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
            r#"Compilation failed!
 = Exit code: {} =
= Stdout =
{}
= Stderr =
{}"#,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

fn run_suite(suite: &Path, provided_out_dir: Option<&Path>, config: &Config) -> Option<NonZeroI32> {
    eprintln!("Value for config: {:?}", config);
    eprintln!("Testing suite: {:?}", suite);

    // let out_dir = env::current_dir().unwrap().join("tmp");
    // fs::create_dir(&out_dir).unwrap_or_default();
    let out_dir = provided_out_dir.map_or_else(env::temp_dir, Path::to_path_buf);
    match lib::run_suite(&suite, &out_dir, &config) {
        Err(err) => {
            match err {
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
                            eprintln!("{}", OutputPrinter(&output));
                        }
                        SuiteDoesNotExist => {
                            eprintln!("{} is not an elm application or package!", suite.display());
                        }

                        OutDirIsNotDir => {
                            match provided_out_dir {
                                Some(dir) =>
                                    eprintln!("{} must either be a directory or a path where elm-torture can create one!",
                                    dir.display()),
                                None =>
                                    panic!("Invalid tempory directory: {}", out_dir.display()),
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
                            panic!("Path was not suite - should have been checked already thogu?");
                        }
                        NodeProcess(err) => {
                            panic!("The node process errored unexpectedly:\n{}", err);
                        }
                        CopyingCustomHarness(err) => {
                            panic!("A custom test harness was found but could not be copied. Details:\n{}", err);
                        }
                        WritingHarness(err) => {
                            panic!(
                                "Cannot add the test harness to the output directory. Details:\n{}",
                                err
                            );
                        }
                        CopyingExpectedOutput(err) => {
                            panic!(
                                "The expected output exists but cannot be copied. Details:\n{}",
                                err
                            );
                        }
                        Runtime(output) => {
                            eprintln!("{}", OutputPrinter(&output));
                        }
                        CannotFindExpectedOutput => {
                            eprintln!("{}\n{}",
                            "Each suite must contain a file 'output.txt', containing the text that",
                            "the suite should write to stdout"
                        );
                        }
                        WrongOutputProduced { actual, expected } => {
                            eprintln!(
                                r#"The suite ran without error but with incorrect output!

= Expected =
{}
= Actual =
{}"#,
                                String::from_utf8_lossy(&actual),
                                String::from_utf8_lossy(&expected),
                            );
                        }
                    }
                    NonZeroI32::new(2)
                }
            }
        }
        Ok(()) => None,
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

fn get_absolute_path_if_possible(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
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
            println!(
                "Running SSCCE {}:",
                get_absolute_path_if_possible(&suite).display()
            );
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
                println!("  {}", get_absolute_path_if_possible(&path).display());
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
