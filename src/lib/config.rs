use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub elm_compiler: String,
    pub node: String,
    pub args: Box<[String]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            elm_compiler: "elm".into(),
            node: "node".into(),
            args: Box::default(),
        }
    }
}
