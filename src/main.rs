#![warn(clippy::all, clippy::pedantic)]

mod lib;

use lib::cli;
use lib::compile_and_run;
use lib::compile_and_run::compile_and_run_suite;
use lib::compile_and_run::compile_and_run_suites;
use lib::compile_and_run::SuiteError;
use lib::formatting;
use std::num::NonZeroI32;
use std::path::PathBuf;
use std::process;

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
                        compile_and_run::Error::Compiler(_) => 0x21,
                        compile_and_run::Error::Runner(_) => 0x22,
                    }
                }
            }

            ExpectedFailure => 0x24,
        },
        Ok(()) => 0,
    }
}

fn run_app(instructions: &cli::Instructions) -> Option<NonZeroI32> {
    let welcome_message = "Elm Torture - stress tests for an elm compiler";
    match instructions.task {
        cli::Task::DumpConfig => {
            println!(
                "{}",
                serde_json::to_string_pretty(&instructions.config)
                    .expect("could not serialize config")
            );
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
            let suite_result =
                compile_and_run_suite(suite, out_dir.as_ref().map(PathBuf::as_ref), &instructions);
            match suite_result {
                Ok(()) => println!(" Success"),
                Err(ref e) => println!("\n\n{}", e),
            }
            NonZeroI32::new(get_exit_code(&suite_result))
        }
        cli::Task::RunSuites(ref suite_dir) => match lib::find_suites::find_suites(&suite_dir) {
            Ok(suites) => {
                let suite_results = compile_and_run_suites(welcome_message, &suites, &instructions);
                let code = suite_results.iter().fold(0, |a, b| a | get_exit_code(&b.1));
                NonZeroI32::new(code)
            }
            Err(ref err) => {
                eprint!("{}", formatting::find_suite_error(err, suite_dir));
                NonZeroI32::new(0x28)
            }
        },
    }
}
fn main() {
    env_logger::init();
    process::exit(run_app(&cli::get_cli_task()).map_or(0, NonZeroI32::get));
}
