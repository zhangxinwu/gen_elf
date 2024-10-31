use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub func_name: String,
}

pub fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}
