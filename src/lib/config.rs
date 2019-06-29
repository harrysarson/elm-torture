use serde::{Deserialize, Serialize};
use std::string::String;


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub program: String,
}


impl Default for Config {
    fn default() -> Self {
        Config {
            program: "elm".into(),
        }
    }
}