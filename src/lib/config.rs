use clap::Clap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::string::String;
use std::time::Duration;

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
    elm_compiler: Option<String>,
    #[clap(long, about = "Path to node.")]
    node: Option<String>,
    #[clap(
        short,
        long,
        about = "Optimization level to use when compiling SSCCEs."
    )]
    opt_level: Option<OptimisationLevel>,
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
            elm_compiler: merge!(elm_compiler),
            node: merge!(node),
            opt_level: merge!(opt_level),
            compiler_max_retries: merge!(compiler_max_retries),
            run_timeout: merge!(run_timeout),
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

    pub fn compiler_max_retries(&self) -> usize {
        self.compiler_max_retries.unwrap_or(1)
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
