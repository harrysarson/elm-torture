use clap::Clap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::string::String;
use std::time::Duration;
use std::{fmt, num::NonZeroI32};

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
        about = "Retry compilation (compile at most <N> times) if it fails."
    )]
    compiler_reruns: Option<NonZeroI32>,
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
        Config {
            elm_compiler: self.elm_compiler,
            node: self.node,
            opt_level: self.opt_level,
            compiler_reruns: self.compiler_reruns,
            run_timeout: self.run_timeout,
        }
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
            compiler_reruns: merge!(compiler_reruns),
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
