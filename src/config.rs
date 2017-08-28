use toml;
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone)]
pub struct Config {
    parsed_config: ServerConfig,
}

#[derive(Deserialize, Clone)]
struct ServerConfig {
    application: String,
    appuser: String,
    printers: Printers,
}

#[derive(Deserialize, Clone)]
struct Printers {
    hosts: Vec<String>,
}

impl Config {
    pub fn new () -> Config {
        let mut file = File::open("Config.toml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let p: ServerConfig = match toml::from_str(&mut contents) {
            Ok(parsed) => parsed,
            Err(e) => panic!("config.rs: Failed to parse Config.toml due to an error: {}", e.to_string()), 
        };

        Config {
            parsed_config: p,
        }
    }

    pub fn get_hosts(&self) -> Result<&Vec<String>, String> {
        if self.parsed_config.printers.hosts.len() > 0 {
            return Ok(&self.parsed_config.printers.hosts);
        } else {
            return Err("Config did not parse correctly or there are no hosts set in Config.toml".to_string());
        }
    }

    pub fn application(&self) -> &String {
        &self.parsed_config.application
    }

    pub fn appuser(&self) -> &String {
        &self.parsed_config.appuser
    }
}

