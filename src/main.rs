#![warn(clippy::all, clippy::pedantic)]

mod lib;


use clap::App;
use clap::Arg;
use lib::compile;
use lib::config;
use lib::config::Config;
use std::env;
use std::fs::File;
use std::num::NonZeroI32;
use std::path::Path;
use std::path::PathBuf;
use std::process;


enum CliTask {
    DumpConfig,
    RunSuite(PathBuf),
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
                .takes_value(true),
        )
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
            CliTask::RunSuite(
                matches
                    .value_of("suite")
                    .unwrap()
                    .parse()
                    .unwrap_or_else(|infalible| match infalible {}),
            )
        },
    }
}

fn run_suite(suite: &Path, config: &Config) -> Option<NonZeroI32> {
    eprintln!("Value for config: {:?}", config);
    eprintln!("Testing suite: {:?}", suite);

    // let out_dir = env::current_dir().unwrap().join("tmp");
    // fs::create_dir(&out_dir).unwrap_or_default();
    let out_dir = env::temp_dir();
    match lib::run_suite(&suite, &out_dir, &config) {
        Err(err) => {
            match err {
                lib::Error::Compiler(err) => {
                    match err {
                        compile::Error::CompilerNotFound(err) => {
                            eprintln!("Could not find elm compiler executable! Details:\n{}", err);
                        }
                        compile::Error::ReadingTargets(err) => {
                            eprintln!("targets.txt found in suite {} but could not be read!. Details:\n{}", &suite.display(), err);
                        }
                        compile::Error::Process(err) => {
                            eprintln!("Failed to execute compiler! Details:\n{}", err);
                        }
                        compile::Error::Compiler(output) => {
                            eprintln!("Compilation failed! Details:\n{:?}", output);
                        }
                        compile::Error::CompilerStdErrNotEmpty(output) => {
                            eprintln!("Compilation sent output to stderr! Details:\n{:?}", output);
                        }
                        compile::Error::SuiteDoesNotExist => {
                            eprintln!("{} is not an elm application or package!", suite.display());
                        }
                    }
                    NonZeroI32::new(1)
                }
                lib::Error::Runner(err) => {
                    dbg!(err);
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
        CliTask::RunSuite(suite) => {
            println!("{}", welcome_message);
            println!();
            println!(
                "Running SSCCE {}:",
                get_absolute_path_if_possible(&suite).display()
            );
            println!();
            run_suite(&suite, &config)
        }
        CliTask::RunSuites(suite_dir) => {
            let suites =
                lib::find_suites::find_suites(suite_dir).expect("error scanning for suites");
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
            iterate_till_some(suites.iter(), |suite| run_suite(suite, &config))
        }
    }
}

fn main() {
    process::exit(run_app(&get_cli_task()).map_or(0, NonZeroI32::get));
}
