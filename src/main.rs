extern crate clap;
extern crate which;
mod lib;

use crate::lib::compile::compile;
use clap::App;
use clap::Arg;

use lib::compile;
use lib::config::Config;

use std::env;

use std::fs::File;
use std::path::Path;
use std::process;


fn main() {
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
            Arg::with_name("project")
                .value_name("DIRECTORY")
                .help("The suite to test")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("show_config")
                .long("showConfig")
                .help("Dump the configuration"),
        )
        .get_matches();


    let config = matches
        .value_of_os("config")
        .map(|p| File::open(p).expect("config file not found"))
        .map(|file| {
            serde_json::from_reader(file).expect("error while reading json configuration file")
        })
        .unwrap_or(Config::default());

    let suite = matches.value_of("project").unwrap();

    let suite = Path::new(suite);

    if matches.is_present("show_config") {
        println!(
            "{}",
            serde_json::to_string_pretty(&config).expect("could not serialize config")
        );
        process::exit(0);
    }

    println!("Value for config: {:?}", config);
    println!("Testing suite: {:?}", suite);

    let out_dir = env::temp_dir();
    if let Err(err) = compile(suite, &out_dir, &config) {
        match err {
            compile::Error::CompilerNotFound(err) => {
                eprintln!("Could not find elm compiler executable! Details:\n{}", err)
            }
            compile::Error::ProcessError(err) => {
                eprintln!("Failed to execute compiler! Details:\n{}", err)
            }
            compile::Error::CompilerError(output) => {
                eprintln!("Compilation failed! Details:\n{:?}", output)
            }
            compile::Error::CompilerStdErrNotEmpty(output) => {
                eprintln!("Compilation sent output to stderr! Details:\n{:?}", output)
            }
        }
        process::exit(1);
    }
}