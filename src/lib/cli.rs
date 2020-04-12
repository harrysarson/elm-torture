use crate::lib::config;
use crate::lib::config::Config;
use clap::App;
use clap::Arg;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

pub enum Task {
    DumpConfig,
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
                .conflicts_with("suites")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("suites")
                .long("suites")
                .value_name("DIRECTORY")
                .help("A directory containing suites to test")
                .conflicts_with("out_dir")
                .takes_value (true))
        .arg(
            Arg::with_name("out_dir")
                .long("out-dir")
                .value_name("DIRECTORY")
                .help("The directory to place built files in.\nMust not exist or be an empty directory")
                .takes_value(true)  )
        .arg(
            Arg::with_name("show_config")
                .long("showConfig")
                .help("Dump the configuration"),
        )
        .arg(
            Arg::with_name("fail_fast")
                .long("fail-fast")
                .help("Stop running on the first failed suite."),
        )
        .get_matches();

    let fail_fast = matches.is_present("fail_fast");

    let config = {
        let config_file = matches.value_of_os("config");

        let mut deserialised: Config = config_file
            .map(|p| File::open(p).expect("config file not found"))
            .map(|file| {
                serde_json::from_reader(file).expect("error while reading json configuration file")
            })
            .unwrap_or_default();

        if let Some(config_dir) = config_file.map(Path::new).and_then(Path::parent) {
            deserialised.allowed_failures = deserialised
                .allowed_failures
                .iter()
                .map(|p| config_dir.join(p))
                .collect();
        }
        deserialised
    };

    Instructions {
        config,
        fail_fast,
        task: if matches.is_present("show_config") {
            Task::DumpConfig
        } else if let Some(suites) = matches.value_of("suites") {
            Task::RunSuites(
                suites
                    .parse()
                    .unwrap_or_else(|infalible| match infalible {}),
            )
        } else {
            Task::RunSuite {
                suite: matches
                    .value_of("suite")
                    .unwrap()
                    .parse()
                    .unwrap_or_else(|infalible| match infalible {}),
                out_dir: matches
                    .value_of("out_dir")
                    .map(|dir| dir.parse().unwrap_or_else(|infalible| match infalible {})),
            }
        },
    }
}
