#![warn(clippy::all, clippy::pedantic)]

mod lib;

use colored::Colorize;
use lib::cli;
use lib::compile;
use lib::run;
use std::fmt;
use std::fs;
use std::io;
use std::mem;
use std::num::NonZeroI32;
use std::path::Path;
use std::path::PathBuf;
use std::process;

fn display_process_output<'a>(output: &'a process::Output) -> impl fmt::Display + 'a {
    lib::easy_format(move |f| {
        write!(
            f,
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
    })
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

fn display_compiler_error<'a>(
    err: &'a compile::Error,
    suite: &'a Path,
    out_dir: &'a OutDir<'a>,
) -> impl fmt::Display + 'a {
    lib::easy_format(move |f| {
        use compile::Error::*;
        match err {
            CompilerNotFound(err) => write!(
                f,
                "Could not find elm compiler executable! Details:\n{}",
                err
            ),
            ReadingTargets(err) => write!(
                f,
                "targets.txt found in suite {} but could not be read!. Details:\n{}",
                suite.display(),
                err
            ),
            Process(err) => panic!("Failed to execute compiler! Details:\n{}", err),
            Compiler(output) | CompilerStdErrNotEmpty(output) => write!(
                f,
                "Compilation failed!\n{}",
                display_process_output(&output)
            ),
            SuiteDoesNotExist => {
                panic!("Path was not suite - this should have been checked already!")
            }

            OutDirIsNotDir => {
                if out_dir.is_tempory() {
                    panic!("Invalid tempory directory: {}", out_dir.path().display())
                } else {
                    write!(
                        f,
                        "{} must either be a directory or a path where elm-torture can create one!",
                        out_dir.path().display()
                    )
                }
            }
        }
    })
}

fn display_runner_error<'a>(err: &'a run::Error, out_dir: &'a Path) -> impl fmt::Display + 'a {
    lib::easy_format(move |f| {
        use run::Error::*;
        match err {
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
                write!(f, "{}", display_process_output(&output))?;
                write!(
                    f,
                    "\n\nTo inspect the built files that caused this error see:\n  {}",
                    out_dir.display()
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
                display_process_output(&output)
            ),
        }
    })
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
                        "Failed to compile suite {}.\n{}\n",
                        &reason.suite.display(),
                        indented::indented(display_compiler_error(
                            &err,
                            &reason.suite,
                            &reason.outdir
                        ))
                    ),

                    lib::Error::Runner(err) => write!(
                        f,
                        "Suite {} failed at run time.\n{}\n",
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
    instructions: &cli::Instructions,
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
    let failure_allowed = instructions.config.allowed_failures.iter().any(|p| {
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
    if instructions.clear_elm_stuff {
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

    let run_result = lib::run_suite(&suite, out_dir.path(), &instructions.config);

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

fn run_suites(
    welcome_message: &str,
    suites: &[PathBuf],
    instructions: &cli::Instructions,
) -> Option<NonZeroI32> {
    assert!(!suites.is_empty());

    println!(
        "{}

Running the following {} SSCCE{}:
{}
",
        welcome_message,
        suites.len(),
        if suites.len() == 1 { "" } else { "s" },
        indented::indented(lib::easy_format(|f| {
            for path in suites.iter() {
                writeln!(f, "{}", path.display())?
            }
            Ok(())
        }))
    );

    let suite_results = {
        let mut tmp = Vec::with_capacity(suites.len());
        for suite in suites {
            let res = run_suite(suite, None, instructions);
            if let Err(ref e) = res {
                println!("{}", e);
            }
            let failed = match res {
                Err(SuiteError::Failure { allowed: true, .. }) | Ok(()) => false,
                Err(_) => true,
            };
            tmp.push((suite, res));
            if instructions.fail_fast && failed {
                break;
            }
        }
        tmp
    };

    println!(
        "
elm-torture has run the following {} SSCCE{}:
{}
",
        suite_results.len(),
        if suite_results.len() == 1 { "" } else { "s" },
        indented::indented(lib::easy_format(|f| {
            for (path, result) in &suite_results {
                writeln!(
                    f,
                    "{} ({})",
                    path.display(),
                    match result {
                        Err(SuiteError::Failure { allowed: true, .. }) =>
                            "allowed failure".yellow(),
                        Err(SuiteError::ExpectedFailure) => "success when failure expected".red(),
                        Err(_) => "failure".red(),
                        Ok(()) => "success".green(),
                    }
                )?
            }
            Ok(())
        }))
    );
    let code = suite_results.iter().fold(0, |a, b| a | get_exit_code(&b.1));
    NonZeroI32::new(code)
}

fn find_suite_error<'a>(
    err: &'a lib::find_suites::Error,
    suite_dir: &'a Path,
) -> impl fmt::Display + 'a {
    lib::easy_format(move |fmt| {
        use lib::find_suites::Error::*;
        match err {
            ProvidedPathIsNotDir => write!(
                fmt,
                "elm-torture cannot run suites in {} as it is not a directory!
    Please check the path and try again.
    ",
                suite_dir.display()
            ),
            ReadingDir(e) => Err(e).unwrap(),
            ProvidedPathIsSuiteItself => write!(
                fmt,
                "elm-torture cannot run suites in {} as it is a suite itself!
To run this suite individually try `--suite {}` (note `suite` rather than `--suites`).
    ",
                suite_dir.display(),
                suite_dir.display(),
            ),
        }
    })
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
                run_suite(suite, out_dir.as_ref().map(PathBuf::as_ref), &instructions);
            match suite_result {
                Ok(()) => println!(" Success"),
                Err(ref e) => println!("\n\n{}", e),
            }
            NonZeroI32::new(get_exit_code(&suite_result))
        }
        cli::Task::RunSuites(ref suite_dir) => match lib::find_suites::find_suites(&suite_dir) {
            Ok(suites) => run_suites(welcome_message, &suites, &instructions),
            Err(ref err) => {
                eprint!("{}", find_suite_error(err, suite_dir));
                NonZeroI32::new(0x28)
            }
        },
    }
}

fn main() {
    env_logger::init();
    process::exit(run_app(&cli::get_cli_task()).map_or(0, NonZeroI32::get));
}
