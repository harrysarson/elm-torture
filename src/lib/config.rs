use clap::Clap;
use serde::{Deserialize, Serialize};
use std::string::String;
use std::time::Duration;
use std::{ffi::OsString, str::FromStr};
use std::{fmt, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clap, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum OptimizationLevel {
    Debug,
    Dev,
    Optimize,
}

impl OptimizationLevel {
    pub fn args(self) -> &'static [&'static str] {
        match self {
            OptimizationLevel::Debug => &["--debug"],
            OptimizationLevel::Dev => &[],
            OptimizationLevel::Optimize => &["--optimize"],
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            OptimizationLevel::Debug => &"debug",
            OptimizationLevel::Dev => &"dev",
            OptimizationLevel::Optimize => &"optimize",
        }
    }
}

impl fmt::Display for OptimizationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OptimizationLevel::Debug => &"debug",
                OptimizationLevel::Dev => &"dev (default)",
                OptimizationLevel::Optimize => &"optimize",
            }
        )
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clap)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[clap(
        long,
        multiple(false),
        use_delimiter(true),
        about = "Path to elm compiler."
    )]
    elm_compilers: Option<Vec<OsString>>,
    #[clap(long, about = "Path to node.")]
    node: Option<String>,
    #[clap(
        short,
        long,
        multiple(false),
        use_delimiter(true),
        about = "Optimization level to use when compiling SSCCEs."
    )]
    opt_levels: Option<Vec<OptimizationLevel>>,
    #[clap(
        long,
        value_name = "N",
        about = "Retry compilation (at most <N> times) if it fails."
    )]
    compiler_max_retries: Option<usize>,
    #[clap(
        long,
        value_name = "DURATION",
        about = "Report run time failure if SSCCE takes more than <DURATION> to run.",
        parse(try_from_str = humantime::parse_duration)
    )]
    run_timeout: Option<Duration>,

    #[clap(
        long,
        value_name = "DIRECTORY",
        about = "The directory to place built files in."
    )]
    pub out_dir: Option<PathBuf>,
}

impl Config {
    pub fn serialize(self) -> impl Serialize {
        self
    }

    pub fn overwrite_with(self, other: Config) -> Config {
        macro_rules! merge {
            ($prop:ident) => {
                other.$prop.or(self.$prop)
            };
        }

        Config {
            elm_compilers: merge!(elm_compilers),
            node: merge!(node),
            opt_levels: merge!(opt_levels),
            compiler_max_retries: merge!(compiler_max_retries),
            run_timeout: merge!(run_timeout),
            out_dir: merge!(out_dir),
        }
    }

    pub fn elm_compilers(&self) -> &Vec<OsString> {
        lazy_static::lazy_static! {
            static ref ELM: Vec<OsString> = vec![OsString::from("elm")];
        }
        &self.elm_compilers.as_ref().unwrap_or(&*ELM)
    }

    pub fn node(&self) -> &str {
        self.node.as_ref().map_or_else(|| "node", String::as_str)
    }

    pub fn opt_levels(&self) -> &[OptimizationLevel] {
        if let Some(levels) = &self.opt_levels {
            &levels
        } else {
            &[OptimizationLevel::Dev]
        }
    }

    pub fn compiler_max_retries(&self) -> usize {
        self.compiler_max_retries.unwrap_or(1)
    }

    pub fn run_timeout(&self) -> Duration {
        self.run_timeout.unwrap_or_else(|| Duration::new(10, 0))
    }
}

#[derive(Debug)]
pub struct InvalidOptimizationLevel(String);

impl FromStr for OptimizationLevel {
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
        write!(f, "Invalid optimization level: {}", self.0)
    }
}
