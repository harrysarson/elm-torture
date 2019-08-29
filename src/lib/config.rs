use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub elm_compiler: String,
    pub node: String,
    pub args: Box<[String]>,
    pub allowed_failures: Box<[PathBuf]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            elm_compiler: "elm".into(),
            node: "node".into(),
            args: Box::default(),
            allowed_failures: Box::default(),
        }
    }
}
