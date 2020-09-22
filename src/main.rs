#![warn(clippy::all, clippy::pedantic)]
// https://github.com/rust-lang/rust-clippy/issues/5822
#![allow(clippy::option_if_let_else)]

mod lib;

use apply::{Also, Apply};
use colored::Colorize;
use lib::cli;
use lib::formatting;
use lib::suite::compile_and_run_suites;
use lib::suite::CompileAndRunError;
use rayon::{iter, prelude::*};
use std::cell::Cell;
use std::{fs, process};
use std::{num::NonZeroI32, path::Path};

const WELCOME_MESSAGE: &str = "Elm Torture - stress tests for an elm compiler";

#[allow(clippy::enum_glob_use)]
fn get_exit_code(suite_result: &Result<(), CompileAndRunError>) -> i32 {
    use CompileAndRunError::*;

    match suite_result {
        Err(ref compile_and_run_error) => match compile_and_run_error {
            CannotDetectStdlibVariant(_)
            | SuiteNotExist
            | SuiteNotDir
            | SuiteNotElm
            | CannotGetSuiteConfig(_) => 0x28,

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
            ExpectedCompileFailure | ExpectedRunFailure => 0x24,
        },
        Ok(()) => 0,
    }
}

// fn collect_and_print()

fn run_suites(
    suites: &[impl AsRef<Path> + Sync],
    instructions: &cli::Instructions,
) -> Option<NonZeroI32> {
    assert!(!suites.is_empty());
    println!(
        "{}

Running the following {} SSCCE{}:
{}
",
        WELCOME_MESSAGE,
        suites.len(),
        if suites.len() == 1 { "" } else { "s" },
        indented::indented(formatting::easy_format(|f| {
            for path in suites.iter() {
                writeln!(f, "{}", path.as_ref().display())?
            }
            Ok(())
        }))
    );

    // let printer = Mutex::new(formatting::PrinterQueue::new());
    let (res_send, res_recv) = crossbeam_channel::bounded(suites.len());
    compile_and_run_suites(suites.par_iter(), instructions)
        .into_par_iter()
        .for_each(|v| res_send.send(v).unwrap());

    let suite_results: Vec<_> = {
        let block = |v: &mut Vec<_>| v.resize_with(suites.len(), || Cell::new(None));
        let suite_results = Vec::new().also(block);
        let mut print_iter = suite_results.iter().peekable();
        for _ in 0..suites.len() {
            let res = res_recv.recv().unwrap();
            let position = suites
                .iter()
                .position(|s| s.as_ref() == res.0.as_ref())
                .unwrap();
            let old_value = suite_results[position].replace(Some(res));
            assert!(old_value.is_none());
            let maybe_printable = print_iter.peek().unwrap();
            maybe_printable
                .replace(None)
                .apply(|printable| {
                    let print_done = if let Some((suite, out_dir, result)) = &printable {
                        if let Err(e) = &result {
                            println!(
                                "{}",
                                indented::indented(formatting::compile_and_run_error(
                                    e,
                                    suite,
                                    &out_dir,
                                    instructions.config.out_dir.as_ref(),
                                ))
                            );
                        };
                        true
                    } else {
                        false
                    };
                    maybe_printable.set(printable);
                    print_done
                })
                .apply(|print_done| {
                    if print_done {
                        print_iter.next().unwrap();
                    }
                });
        }
        suite_results
            .into_iter()
            .map(|r| r.into_inner().unwrap())
            .collect()
    };

    // .inspect(|(suite, out_dir, res)| {
    //     let index = suites
    //         .iter()
    //         .position(|some_suite| some_suite.as_ref() == suite.as_ref())
    //         .unwrap();
    //     if let Err(e) = res {
    //         printer.lock().unwrap().add_printable(
    //             index,
    //             format!(
    //                 "{}",
    //                 indented::indented(formatting::compile_and_run_error(
    //                     e,
    //                     suite,
    //                     out_dir,
    //                     instructions.config.out_dir.as_ref(),
    //                 ))
    //             ),
    //         );
    //     } else {
    //         printer.lock().unwrap().skip(index)
    //     }
    // })
    // .collect();

    println!(
        "
elm-torture has run the following {} SSCCE{}:
{}
",
        suites.len(),
        if suites.len() == 1 { "" } else { "s" },
        indented::indented(formatting::easy_format(|f| {
            for (suite, _, result) in &suite_results {
                writeln!(
                    f,
                    "{} ({})",
                    suite.as_ref().display(),
                    match result {
                        Err(CompileAndRunError::RunFailure { allowed: true, .. }) =>
                            "allowed run failure".yellow(),
                        Err(CompileAndRunError::CompileFailure { allowed: true, .. }) =>
                            "allowed compile failure".yellow(),
                        Err(CompileAndRunError::ExpectedCompileFailure) =>
                            "success when elm-torture expected a compile time failure".red(),
                        Err(CompileAndRunError::ExpectedRunFailure) =>
                            "success when elm-torture expected a run time failure".red(),
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
        .fold(0, |code, (_, _, res)| code | get_exit_code(res));
    NonZeroI32::new(code)
}

fn run_app(instructions: cli::Instructions) -> Option<NonZeroI32> {
    match &instructions.task {
        cli::Task::DumpConfig(config_file) => {
            let file = fs::File::create(config_file).expect("could create config file");
            serde_json::to_writer_pretty(file, &instructions.config.serialize())
                .expect("could not serialize config");
            None
        }
        cli::Task::RunSuite { ref suite } => {
            print!(
                "{}

Running SSCCE {}:",
                WELCOME_MESSAGE,
                suite.display()
            );
            let (suite2, sscce_out_dir, suite_result) =
                compile_and_run_suites(iter::once(suite), &instructions)
                    .into_par_iter()
                    .collect::<Vec<_>>()
                    .apply(|mut v| {
                        assert!(v.len() == 1);
                        v.pop().unwrap()
                    });
            assert!(suite2 == suite);
            match suite_result {
                Ok(()) => println!(" Success"),
                Err(ref e) => {
                    println!(
                        "\n\n{}",
                        formatting::compile_and_run_error(
                            e,
                            suite,
                            sscce_out_dir,
                            instructions.config.out_dir.as_ref()
                        )
                    );
                }
            }
            NonZeroI32::new(get_exit_code(&suite_result))
        }
        cli::Task::RunSuites(ref suite_dir) => match lib::find_suites::find_suites(&suite_dir) {
            Ok(suites) => run_suites(&suites, &instructions),
            Err(ref err) => {
                eprint!("{}", formatting::find_suite_error(err, suite_dir));
                NonZeroI32::new(0x28)
            }
        },
    }
}

fn main() {
    env_logger::init();
    process::exit(run_app(cli::get_cli_task()).map_or(0, NonZeroI32::get));
}
