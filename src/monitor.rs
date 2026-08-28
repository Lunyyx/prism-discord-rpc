use log::{error, info, warn};
use serde::Deserialize;
use std::fs;
use std::path::{Path};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;
use crate::discord::DiscordClient;
use crate::models::{Instance, Session};

#[derive(Deserialize)]
pub struct MmcPack {
    components: Vec<Component>,
}

#[derive(Deserialize)]
pub struct Component {
    uid: String,
    version: Option<String>,
}

enum SessionEvent {
    Started,
    Stopped,
    None,
}

fn update_session(session: &mut Option<Session>, instance: Option<Instance>) -> SessionEvent {
    match instance {
        Some(instance) => {
            if session.is_none() {
                info!("Minecraft started !");
                info!("Instance : {}", instance.name);
                info!("Minecraft : {}", instance.minecraft_version);

                *session = Some(Session { instance });

                return SessionEvent::Started;
            }

            SessionEvent::None
        }

        None => {
            if session.is_some() {
                *session = None;
                return SessionEvent::Stopped;
            }

            SessionEvent::None
        }
    }
}

pub fn monitor() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::new_all();
    let mut session: Option<Session> = None;

    let mut discord: Option<DiscordClient> = None;
    let mut next_reconnect = Instant::now();
    let mut next_activity_update = Instant::now();

    let mut discord_connection_failed = false;

    loop {
        system.refresh_all();

        if discord.is_none() && Instant::now() >= next_reconnect {
            match DiscordClient::connect() {
                Ok(client) => {
                    discord = Some(client);
                    next_activity_update = Instant::now() + Duration::from_secs(30);

                    if let Some(session) = session.as_ref() {
                        let result = if let Some(discord_client) = discord.as_mut() {
                            discord_client.update_from_session(session)
                        } else {
                            Ok(())
                        };

                        if let Err(error) = result {
                            warn!("Discord activity update failed: {}", error);

                            discord = None;
                            next_reconnect = Instant::now() + Duration::from_secs(5);
                        }
                    }
                }

                Err(error) => {
                    if !discord_connection_failed {
                        warn!("Discord connection failed: {}", error);
                        discord_connection_failed = true;
                    }                    
                    
                    next_reconnect = Instant::now() + Duration::from_secs(5);
                }
            }
        }

        let instance = find_instance(&system);

        let event = update_session(&mut session, instance);

        if Instant::now() >= next_activity_update {
            if let Some(session) = session.as_ref() {
                let result = if let Some(discord_client) = discord.as_mut() {
                    discord_client.update_from_session(session)
                } else {
                    Ok(())
                };

                if let Err(error) = result {
                    warn!("Discord connection lost: {}", error);

                    discord = None;
                    next_reconnect = Instant::now() + Duration::from_secs(5);
                }
            }

            next_activity_update = Instant::now() + Duration::from_secs(30);
        }

        match event {
            SessionEvent::Started => {
                if let Some(session) = session.as_ref() {
                    let result = if let Some(discord_client) = discord.as_mut() {
                        discord_client.update_from_session(session)
                    } else {
                        Ok(())
                    };

                    if let Err(error) = result {
                        warn!("Discord connection lost: {}", error);

                        discord = None;
                        next_reconnect = Instant::now() + Duration::from_secs(5);
                    }
                }
            }

            SessionEvent::Stopped => {
                if let Some(discord_client) = discord.as_mut() {
                    if let Err(error) = discord_client.clear_activity() {
                        warn!("Discord connection lost: {}", error);

                        discord = None;
                        next_reconnect = Instant::now() + Duration::from_secs(5);
                    }
                }
            }

            SessionEvent::None => {}
        }

        thread::sleep(Duration::from_secs(2));
    }
}

fn find_instance(system: &System) -> Option<Instance> {
    for (_pid, process) in system.processes() {
        let name = process.name().to_string_lossy();
        let command = process.cmd();

        if name == "java"
            && command
                .iter()
                .any(|arg| arg == "org.prismlauncher.EntryPoint")
        {
            for arg in command {
                let arg = arg.to_string_lossy();

                if let Some(path) = arg.strip_prefix("-Djava.library.path=") {
                    if let Some(instance_path) = Path::new(path).parent() {
                        match read_instance(instance_path) {
                            Ok(instance) => {
                                return Some(instance);
                            }
                            Err(error) => {
                                error!("Error : {}", error);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn read_instance(path: &Path) -> Result<Instance, Box<dyn std::error::Error>> {
    let config_path = path.join("instance.cfg");
    let config = fs::read_to_string(&config_path)?;

    let mut name = String::from("Unknown");

    for line in config.lines() {
        if let Some(value) = line.strip_prefix("name=") {
            name = value.to_string();
        }
    }

    let minecraft_version = read_minecraft_version(path)?;

    Ok(Instance {
        name,
        minecraft_version,
    })
}

fn read_minecraft_version(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let pack_path = path.join("mmc-pack.json");

    let content = fs::read_to_string(pack_path)?;

    let pack: MmcPack = serde_json::from_str(&content)?;

    for component in pack.components {
        if component.uid == "net.minecraft" {
            if let Some(version) = component.version {
                return Ok(version);
            }

            return Err("Minecraft version not found".into());
        }
    }

    Err("Minecraft component not found".into())
}