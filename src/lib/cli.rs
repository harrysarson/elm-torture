use crate::lib::config;
use clap::Clap;
use std::{ffi::OsStr, path::PathBuf};
use std::{fs::File, path::Path};

#[derive(Clap)]
#[clap(
    version = "0.0.2",
    author = "Harry Sarson <harry.sarson@hotmail.co.uk>",
    about = "Test suite for an elm compiler",
    group=clap::ArgGroup::new("suite_or_suites").required(true)
)]
struct Opts {
    #[clap(short, long = "config", about = "Set config file", parse(try_from_os_str = read_config_file))]
    config_from_file: Option<config::Config>,

    #[clap(flatten)]
    config: config::Config,

    #[clap(
        long,
        value_name = "DIRECTORY",
        about = "The suite to test",
        group = "suite_or_suites"
    )]
    suite: Option<PathBuf>,

    #[clap(
        long,
        value_name = "DIRECTORY",
        about = "A directory containing suites to test",
        group = "suite_or_suites",
        conflicts_with = "out-dir"
    )]
    suites: Option<PathBuf>,

    #[clap(
        long,
        value_name = "DIRECTORY",
        about = "The directory to place built files in."
    )]
    out_dir: Option<PathBuf>,

    #[clap(long, value_name = "FILE", about = "Dump the configuration to FILE.")]
    show_config: Option<PathBuf>,

    #[clap(long, about = "Stop running on the first failed suite.")]
    fail_fast: bool,
}

pub enum Task {
    DumpConfig(PathBuf),
    RunSuite {
        suite: PathBuf,
        out_dir: Option<PathBuf>,
    },
    RunSuites(PathBuf),
}

pub struct Instructions {
    pub config: config::Config,
    pub fail_fast: bool,
    pub task: Task,
}

fn read_config_file(config_path: &OsStr) -> Result<config::Config, String> {
    let file = File::open(config_path).map_err(|e| {
        format!(
            "Config file {} not found: {}",
            AsRef::<Path>::as_ref(config_path).to_string_lossy(),
            e
        )
    })?;
    serde_json::from_reader(file).map_err(|e| {
        format!(
            "Could not parse file {} as json config: {}",
            AsRef::<Path>::as_ref(config_path).to_string_lossy(),
            e
        )
    })
}

pub fn get_cli_task() -> Instructions {
    let Opts {
        suite,
        suites,
        out_dir,
        config_from_file,
        fail_fast,
        show_config,
        config: config_from_cli,
        ..
    } = Opts::parse();

    let config = if let Some(c) = config_from_file {
        c.overwrite_with(config_from_cli)
    } else {
        config_from_cli
    };

    Instructions {
        config,
        fail_fast,
        task: show_config.map_or_else(
            || {
                suites.map_or_else(
                    || Task::RunSuite {
                        suite: suite.unwrap(),
                        out_dir,
                    },
                    Task::RunSuites,
                )
            },
            Task::DumpConfig,
        ),
    }
}
