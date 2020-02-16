use super::compile;
use super::compile_and_run;
use super::compile_and_run::SuiteError;
use super::find_suites;
use super::run;
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

fn compiler_error<'a>(
    err: &'a compile::Error,
    suite: &'a Path,
    out_dir: &'a compile_and_run::OutDir<'a>,
) -> impl fmt::Display + 'a {
    easy_format(move |f| {
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
            Compiler(output) | CompilerStdErrNotEmpty(output) => {
                write!(f, "Compilation failed!\n{}", process_output(&output))
            }
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

fn runner_error<'a>(err: &'a run::Error, out_dir: &'a Path) -> impl fmt::Display + 'a {
    easy_format(move |f| {
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
            OutputProduced(output) => write!(
                f,
                "The suite ran without error but produced the following output!:\n{}",
                process_output(&output)
            ),
        }
    })
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
                    compile_and_run::Error::Compiler(err) => write!(
                        f,
                        "Failed to compile suite {}.\n{}\n",
                        &reason.suite.display(),
                        indented::indented(compiler_error(&err, &reason.suite, &reason.outdir))
                    ),

                    compile_and_run::Error::Runner(err) => write!(
                        f,
                        "Suite {} failed at run time.\n{}\n",
                        &reason.suite.display(),
                        indented::indented(runner_error(&err, reason.outdir.path()))
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
