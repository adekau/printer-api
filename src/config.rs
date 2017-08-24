use toml;
use std::fs::File;
use std::io::prelude::*;

pub struct Config {
    toml: String,
}

#[derive(Deserialize)]
struct ServerConfig {
    printers: Printers,
}

#[derive(Deserialize)]
struct Printers {
    hosts: Vec<String>,
}

impl Config {
    pub fn new () -> Config {
        let mut file = File::open("Config.toml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        Config {
            toml: contents,
        }
        
        // if let Err(e) = file.read_to_string(&toml) {
        //     println!("Encountered an error in get_hosts(): {}", e.to_string());
        //     return Err("Failed to read file to string.".to_string())
        // };
    }

    pub fn get_hosts (&mut self) -> Result<Vec<String>, String> {
        let p: ServerConfig = toml::from_str(&mut self.toml).unwrap();

        Ok(p.printers.hosts)
    }
}

