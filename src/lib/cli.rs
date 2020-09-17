use crate::lib::config;
use clap::Clap;
use std::fs::File;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(
    version = "0.0.2",
    author = "Harry Sarson <harry.sarson@hotmail.co.uk>",
    about = "Test suite for an elm compiler",
    group=clap::ArgGroup::new("suite_or_suites").required(true)
)]
struct Opts {
    #[clap(short, long = "config", about = "Set config file")]
    config_path: Option<PathBuf>,

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

pub fn get_cli_task() -> Instructions {
    let Opts {
        suite,
        suites,
        out_dir,
        config_path,
        fail_fast,
        show_config,
        config: config_from_cli,
        ..
    } = Opts::parse();

    let config = {
        if let Some(config_from_file) = config_path.map(|p| -> config::Config {
            let file = File::open(&p).expect("config file not found");
            serde_json::from_reader(file).expect("error while reading json configuration file")
        }) {
            config_from_file.overwrite_with(config_from_cli)
        } else {
            config_from_cli
        }
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
