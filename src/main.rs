#![warn(clippy::all, clippy::pedantic)]
// Due to some bug in async desuging we get false positives
#![allow(clippy::used_underscore_binding)]

mod lib;

use colored::Colorize;
use futures::stream;
use futures::stream::StreamExt;
use lib::cli;
use lib::formatting;
use lib::suite::compile_and_run;
use lib::suite::compile_and_run_suites;
use lib::suite::CompileAndRunError;
use std::num::NonZeroI32;
use std::{fs, process};

#[allow(clippy::enum_glob_use)]
fn get_exit_code<P>(suite_result: &Result<(), CompileAndRunError<P>>) -> i32 {
    use CompileAndRunError::*;

    match suite_result {
        Err(ref compile_and_run_error) => match compile_and_run_error {
            SuiteNotExist | SuiteNotDir | SuiteNotElm => 0x28,

            CompileFailure { allowed, .. } => {
                if *allowed {
                    0
                } else {
                    0x21
                }
            }

            RunFailure { allowed, .. } => {
                if *allowed {
                    0
                } else {
                    0x22
                }
            }

            ExpectedFailure => 0x24,
        },
        Ok(()) => 0,
    }
}

async fn run_app(instructions: cli::Instructions) -> Option<NonZeroI32> {
    let welcome_message = "Elm Torture - stress tests for an elm compiler";
    match &instructions.task {
        cli::Task::DumpConfig(config_file) => {
            let file = fs::File::create(config_file).expect("could create config file");
            serde_json::to_writer_pretty(file, &instructions.config.serialize(config_file))
                .expect("could not serialize config");
            None
        }
        cli::Task::RunSuite {
            ref suite,
            ref out_dir,
        } => {
            print!(
                "{}

Running SSCCE {}:",
                welcome_message,
                suite.display()
            );
            let suite_result = compile_and_run(suite, out_dir.as_ref(), &instructions).await;
            match suite_result {
                Ok(()) => println!(" Success"),
                Err(ref e) => println!(
                    "\n\n{}",
                    formatting::compile_and_run_error(e, suite, out_dir.as_ref())
                ),
            }
            NonZeroI32::new(get_exit_code(&suite_result))
        }
        cli::Task::RunSuites(ref suite_dir) => match lib::find_suites::find_suites(&suite_dir) {
            Ok(suites) => {
                assert!(!suites.is_empty());
                println!(
                    "{}

Running the following {} SSCCE{}:
{}
",
                    welcome_message,
                    suites.len(),
                    if suites.len() == 1 { "" } else { "s" },
                    indented::indented(formatting::easy_format(|f| {
                        for path in suites.iter() {
                            writeln!(f, "{}", path.display())?
                        }
                        Ok(())
                    }))
                );

                let suite_results: Vec<_> =
                    compile_and_run_suites(stream::iter(suites.iter()), &instructions)
                        .collect()
                        .await;
                println!(
                    "
elm-torture has run the following {} SSCCE{}:
{}
",
                    suites.len(),
                    if suites.len() == 1 { "" } else { "s" },
                    indented::indented(formatting::easy_format(|f| {
                        for (suite, result) in &suite_results {
                            writeln!(
                                f,
                                "{} ({})",
                                suite.display(),
                                match result {
                                    Err(CompileAndRunError::RunFailure {
                                        allowed: true, ..
                                    }) => "allowed run failure".yellow(),
                                    Err(CompileAndRunError::CompileFailure {
                                        allowed: true,
                                        ..
                                    }) => "allowed compile failure".yellow(),
                                    Err(CompileAndRunError::ExpectedFailure) =>
                                        "success when failure expected".red(),
                                    Err(_) => "failure".red(),
                                    Ok(_) => "success".green(),
                                }
                            )?
                        }
                        Ok(())
                    }))
                );
                let code = suite_results
                    .iter()
                    .fold(0, |code, (_, res)| code | get_exit_code(res));
                NonZeroI32::new(code)
            }
            Err(ref err) => {
                eprint!("{}", formatting::find_suite_error(err, suite_dir));
                NonZeroI32::new(0x28)
            }
        },
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    process::exit(
        run_app(cli::get_cli_task())
            .await
            .map_or(0, NonZeroI32::get),
    );
}
