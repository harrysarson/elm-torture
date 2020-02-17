#![warn(clippy::all, clippy::pedantic)]

mod lib;

use colored::Colorize;
use lib::cli;
use lib::formatting;
use lib::run;
use lib::run::compile_and_run_suite;
use lib::run::compile_and_run_suites;
use lib::run::SuiteError;
use std::num::NonZeroI32;
use std::process;

fn get_exit_code<P>(suite_result: &Result<(), SuiteError<P>>) -> i32 {
    use SuiteError::*;

    match suite_result {
        Err(ref suite_error) => match suite_error {
            SuiteNotExist | SuiteNotDir | SuiteNotElm => 0x28,

            Failure {
                reason, allowed, ..
            } => {
                if *allowed {
                    0
                } else {
                    match reason {
                        run::CompileAndRunError::Compiler(_) => 0x21,
                        run::CompileAndRunError::Runner(_) => 0x22,
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
            let suite_result = compile_and_run_suite(suite, out_dir.as_ref(), &instructions);
            match suite_result {
                Ok(()) => println!(" Success"),
                Err(ref e) => println!("\n\n{}", formatting::suite_error(e, suite)),
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
                    compile_and_run_suites(suites.iter(), &instructions).collect();
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
                                    Err(SuiteError::Failure { allowed: true, .. }) =>
                                        "allowed failure".yellow(),
                                    Err(SuiteError::ExpectedFailure) =>
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
fn main() {
    env_logger::init();
    process::exit(run_app(&cli::get_cli_task()).map_or(0, NonZeroI32::get));
}
