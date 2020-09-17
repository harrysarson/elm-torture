use clap::Clap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::string::String;
use std::time::Duration;
use std::{fmt, num::NonZeroI32};
use std::{path::Path, str::FromStr};

// fn xxx() -> {

//     if let Some(config_dir) = config_file.map(Path::new).and_then(Path::parent) {
//         deserialised.allowed_failures = deserialised
//             .allowed_failures()
//             .iter()
//             .map(|p| config_dir.join(p))
//             .collect();
//     }
// }

#[derive(Debug, Deserialize, Serialize, Clap, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum OptimisationLevel {
    Debug,
    Dev,
    Optimize,
}

#[derive(Debug, Default, Deserialize, Serialize, Clap)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[clap(long, about = "Path to elm compiler.")]
    pub elm_compiler: Option<String>,
    #[clap(long, about = "Path to node.")]
    pub node: Option<String>,
    #[clap(
        short,
        long,
        about = "Optimization level to use when compiling SSCCEs."
    )]
    pub opt_level: Option<OptimisationLevel>,
    #[clap(
        long,
        value_name = "N",
        about = "Retry compilation (compile at most <N> times) if it fails."
    )]
    pub compiler_reruns: Option<NonZeroI32>,
    #[clap(
        long,
        value_name = "DURATION",
        about = "Report run time failure if SSCCE takes more than <DURATION> to run.",
        parse(try_from_str = humantime::parse_duration)
    )]
    pub run_timeout: Option<Duration>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RelativePath(PathBuf);

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<RelativePath> for RelativePath {
    fn as_ref(&self) -> &RelativePath {
        &self
    }
}

// impl Config<RelativePath> {
//     pub fn into_config<P>(self, config_file_location: P) -> Config<PathBuf>
//     where
//         P: AsRef<Path>,
//     {
//         let allowed_failures = self.allowed_failures.map(|allowed_failures| {
//             allowed_failures
//                 .into_vec()
//                 .into_iter()
//                 .map(|file_path: RelativePath| {
//                     config_file_location
//                         .as_ref()
//                         .parent()
//                         .map(|dirname| dirname.join(&file_path))
//                         .unwrap_or(file_path.0)
//                 })
//                 .collect()
//         });
//         Config {
//             elm_compiler: self.elm_compiler,
//             node: self.node,
//             args: self.args,
//             allowed_failures,
//             compiler_reruns: self.compiler_reruns,
//             run_timeout: self.run_timeout,
//         }
//     }
// }

impl Config {
    pub fn serialize(self) -> impl Serialize {
        Config {
            elm_compiler: self.elm_compiler,
            node: self.node,
            opt_level: self.opt_level,
            compiler_reruns: self.compiler_reruns,
            run_timeout: self.run_timeout,
        }
    }

    pub fn elm_compiler(&self) -> &str {
        self.elm_compiler
            .as_ref()
            .map_or_else(|| "elm", String::as_str)
    }

    pub fn node(&self) -> &str {
        self.node.as_ref().map_or_else(|| "node", String::as_str)
    }

    pub fn opt_level(&self) -> OptimisationLevel {
        self.opt_level.unwrap_or(OptimisationLevel::Dev)
    }

    pub fn args(&self) -> &[&'static str] {
        match self.opt_level() {
            OptimisationLevel::Debug => &["--debug"],
            OptimisationLevel::Dev => &[],
            OptimisationLevel::Optimize => &["--optimize"],
        }
    }

    pub fn compiler_reruns(&self) -> NonZeroI32 {
        self.compiler_reruns
            .unwrap_or_else(|| NonZeroI32::new(1).unwrap())
    }

    pub fn run_timeout(&self) -> Duration {
        self.run_timeout.unwrap_or_else(|| Duration::new(10, 0))
    }
}

#[derive(Debug)]
pub struct InvalidOptimizationLevel(String);

impl FromStr for OptimisationLevel {
    type Err = InvalidOptimizationLevel;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "debug" => Self::Debug,
            "dev" => Self::Dev,
            "optimize" => Self::Optimize,
            _ => return Err(InvalidOptimizationLevel(s.to_string())),
        })
    }
}

impl fmt::Display for InvalidOptimizationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid optimisation level: {}", self.0)
    }
}
