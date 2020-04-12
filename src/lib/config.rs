use serde::{Deserialize, Serialize};
use std::num::NonZeroI32;
use std::path::PathBuf;
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub elm_compiler: String,
    pub node: String,
    pub args: Box<[String]>,
    pub allowed_failures: Box<[PathBuf]>,
    pub compiler_reruns: NonZeroI32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            elm_compiler: "elm".into(),
            node: "node".into(),
            args: Box::default(),
            allowed_failures: Box::default(),
            compiler_reruns: NonZeroI32::new(1).unwrap(),
        }
    }
}
