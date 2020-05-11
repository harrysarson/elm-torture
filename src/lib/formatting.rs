#![allow(clippy::enum_glob_use)]

use super::find_suites;
use super::suite;
use super::suite::CompileAndRunError;
use std::fmt;
use std::path::Path;
use std::process;

pub fn easy_format<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result>(func: F) -> impl fmt::Display {
    struct Formatable<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> {
        func: F,
    }
    impl<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> fmt::Display for Formatable<F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.func)(f)
        }
    }
    Formatable { func }
}

fn process_output<'a>(output: &'a process::Output) -> impl fmt::Display + 'a {
    easy_format(move |f| {
        write!(
            f,
            r#"
 = Exit code: {} =
{}
{}"#,
            output.status,
            process_stdout(&output.stdout),
            process_stderr(&output.stderr)
        )
    })
}

fn process_stdout<'a>(stdout: &'a [u8]) -> impl fmt::Display + 'a {
    easy_format(move |f| {
        write!(
            f,
            r#" = Std Out =
{}"#,
            String::from_utf8_lossy(stdout)
        )
    })
}

fn process_stderr<'a>(stderr: &'a [u8]) -> impl fmt::Display + 'a {
    easy_format(move |f| {
        write!(
            f,
            r#" = Std Err =
{}"#,
            String::from_utf8_lossy(stderr)
        )
    })
}

fn compiler_error<'a, P1: AsRef<Path> + 'a, P2: AsRef<Path> + 'a>(
    err: &'a suite::CompileError,
    suite: P1,
    out_dir: Option<P2>,
) -> impl fmt::Display + 'a {
    easy_format(move |f| {
        use suite::CompileError::*;
        match err {
            CompilerNotFound(err) => write!(
                f,
                "Could not find elm compiler executable! Details:\n{}",
                err
            ),
            ReadingTargets(err) => write!(
                f,
                "targets.txt found in suite {} but could not be read!. Details:\n{}",
                suite.as_ref().display(),
                err
            ),
            Process(err) => panic!("Failed to execute compiler! Details:\n{}", err),
            Compiler(output) | CompilerStdErrNotEmpty(output) => {
                write!(f, "Compilation failed!\n{}", process_output(&output))
            }
            SuiteDoesNotExist => {
                panic!("Path was not suite - this should have been checked already!")
            }

            OutDirIsNotDir => {
                if let Some(dir) = out_dir.as_ref() {
                    write!(
                        f,
                        "{} must either be a directory or a path where elm-torture can create one!",
                        dir.as_ref().display()
                    )
                } else {
                    panic!("Invalid tempory directory")
                }
            }
        }
    })
}

fn run_error<'a>(err: &'a suite::RunError, out_dir: &'a Path) -> impl fmt::Display + 'a {
    easy_format(move |f| {
        use suite::RunError::*;
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
                write!(f, "{}", process_output(&output))?;
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
            ExpectedOutputPathNotUtf8(p) => write!(
                f,
                "The canonical path to the expected output json file ({}) is not valid utf8",
                p.display()
            ),
            OutputProduced(output) => write!(
                f,
                "The suite ran without error but produced the following output!:\n{}",
                process_output(&output)
            ),
            Timeout {
                after,
                stdout,
                stderr,
            } => write!(
                f,
                "Running of the suite was stopped after {}.{}

To inspect the built files that caused this error see: {}",
                humantime::format_duration(*after),
                easy_format(|f| {
                    if !stdout.is_empty() || !stderr.is_empty() {
                        write!(
                            f,
                            " Before it stopped the process procuduced the following output:
{}
{}",
                            process_stdout(stdout),
                            process_stderr(stderr)
                        )
                    } else {
                        write!(f, "(The process prouduced no output)")
                    }
                }),
                out_dir.display()
            ),
        }
    })
}

pub fn compile_and_run_error<'a, Pe: AsRef<Path>, Pp: AsRef<Path> + 'a, Ps: AsRef<Path> + 'a>(
    err: &'a CompileAndRunError<Pe>,
    suite: Ps,
    provided_path: Option<Pp>,
) -> impl fmt::Display + 'a {
    easy_format(move |f| {
        use CompileAndRunError::*;

        match err {
            SuiteNotExist => write!(
                f,
                "The provided path to a suite: \"{}\"  does not exist",
                suite.as_ref().display()
            ),

            SuiteNotDir => write!(
                f,
                "The provided path to a suite: \"{}\" exists but is not a directory",
                suite.as_ref().display()
            ),

            SuiteNotElm => write!(
                f,
                "The suite directory: \"{}\" is not an Elm application or package",
                suite.as_ref().display()
            ),

            CompileFailure { allowed, reason } => {
                write!(
                    f,
                    "Failed to compile suite {}.\n{}\n",
                    &suite.as_ref().display(),
                    indented::indented(compiler_error(&reason, &suite, provided_path.as_ref()))
                )?;
                if *allowed {
                    write!(f, "Failure allowed, continuing...")
                } else {
                    Ok(())
                }
            }

            RunFailure {
                allowed,
                outdir,
                reason,
            } => {
                write!(
                    f,
                    "Suite {} failed at run time.\n{}\n",
                    &suite.as_ref().display(),
                    indented::indented(run_error(&reason, outdir.path()))
                )?;
                if *allowed {
                    write!(f, "Failure allowed, continuing...")
                } else {
                    Ok(())
                }
            }

            ExpectedFailure => write!(
                f,
                "Suite {} was expected to fail but did not!",
                &suite.as_ref().display(),
            ),
        }
    })
}

pub fn find_suite_error<'a>(
    err: &'a find_suites::Error,
    suite_dir: &'a Path,
) -> impl fmt::Display + 'a {
    easy_format(move |fmt| {
        use find_suites::Error::*;
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
