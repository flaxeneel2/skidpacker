use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub license: String,
    pub input_jar: String,
    pub threads: usize,
    pub verbose: bool
}

impl Config {
    /// Generate the config file.
    /// Values taken from default.
    /// # Arguments
    /// * `cfg_path` - Path of the config
    fn generate(cfg_path: &str) {
        let data = Self::default();
        fs::write(cfg_path, serde_yaml::to_string(&data).unwrap()).expect("Config generation failed!");
    }
    /// Load a file from a given path
    /// # Arguments
    /// * `cfg_path` - Path to load the config file from
    pub fn load(cfg_path: &str) -> Self {
        if !Path::new(cfg_path).exists() {
            println!("Config file not found! Generating config file!");
            Self::generate(cfg_path);
        }
        let cfg: Config = serde_yaml::from_str(fs::read_to_string(cfg_path).unwrap().as_str()).unwrap();
        cfg
    }

}

impl Default for Config {
    /// Defaults for the config file.
    /// Used for the generation of the config file.
    fn default() -> Self {
        Config {
            license: "PLEASE PUT YOUR LICENSE HERE".to_string(),
            input_jar: "PLEASE ENTER INPUT JAR NAME/PATH".to_string(),
            threads: 4,
            verbose: false
        }
    }
}