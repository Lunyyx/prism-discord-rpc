use std::fs;
use std::sync::OnceLock;

use log::info;
use serde::{Deserialize, Serialize};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub discord_activity: DiscordActivityConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordActivityConfig {
    pub name: String,
    pub details: String,
    pub state: String,
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing config...");

    let config = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("prism-discord-rpc");

    std::fs::create_dir_all(&config).unwrap();

    let config_file = config.join("config.toml");

    if !config_file.exists() {
        info!("Creating default config.toml file...");
        std::fs::File::create(&config_file).unwrap();
    
        let default_config = Config { 
            discord_activity: DiscordActivityConfig {
                name: "Minecraft".to_string(), 
                details: "{{ minecraft_version }}".to_string(),
                state: "{{ instance_name }}".to_string(),
            } 
        };

        let serialized_default_config = toml::to_string(&default_config).unwrap();

        fs::write(config_file, serialized_default_config).unwrap();
    }

    info!("Initialization done");

    Ok(())
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let config_file_content = fs::read_to_string(dirs::config_dir().unwrap().join("prism-discord-rpc").join("config.toml"))?;
    
    let deserialized_config: Config = toml::from_str(&config_file_content).unwrap();

    Ok(deserialized_config)
}