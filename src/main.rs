use log::{error, info, warn};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

const DISCORD_CLIENT_ID: &str = "1538515152788258837";

#[derive(Deserialize)]
struct MmcPack {
    components: Vec<Component>,
}

#[derive(Deserialize)]
struct Component {
    uid: String,
    version: String,
}

struct Instance {
    name: String,
    minecraft_version: String,
}

struct Session {
    instance: Instance,
}

struct DiscordClient {
    socket: UnixStream,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .init();

    monitor()
}

impl DiscordClient {
    fn update_from_session(&mut self, session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        let state = format!("Minecraft {}", session.instance.minecraft_version);

        self.set_activity("Minecraft", &session.instance.name, &state)
    }

    fn clear_activity(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": null
            },
            "nonce": uuid::Uuid::new_v4().to_string()
        });

        Self::send(&mut self.socket, 1, &payload.to_string())?;

        let _ = Self::receive(&mut self.socket)?;

        Ok(())
    }

    fn set_activity(
        &mut self,
        name: &str,
        details: &str,
        state: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let activity = serde_json::json!({
            "name": name,
            "details": details,
            "state": state,
        });

        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": activity
            },
            "nonce": uuid::Uuid::new_v4().to_string()
        });

        Self::send(&mut self.socket, 1, &payload.to_string())?;

        let _ = Self::receive(&mut self.socket)?;

        Ok(())
    }

    fn receive(socket: &mut UnixStream) -> Result<(u32, String), Box<dyn std::error::Error>> {
        let mut opcode_bytes = [0u8; 4];
        let mut length_bytes = [0u8; 4];

        socket.read_exact(&mut opcode_bytes)?;
        socket.read_exact(&mut length_bytes)?;

        let opcode = u32::from_le_bytes(opcode_bytes);
        let length = u32::from_le_bytes(length_bytes) as usize;

        let mut payload = vec![0u8; length];

        socket.read_exact(&mut payload)?;

        let payload = String::from_utf8(payload)?;

        Ok((opcode, payload))
    }

    fn send(
        socket: &mut UnixStream,
        opcode: u32,
        payload: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = payload.as_bytes();
        let length = payload.len() as u32;

        socket.write_all(&opcode.to_le_bytes())?;
        socket.write_all(&length.to_le_bytes())?;
        socket.write_all(payload)?;

        Ok(())
    }

    fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

        for i in 0..10 {
            let socket_path = format!("{}/discord-ipc-{}", runtime_dir, i);

            match UnixStream::connect(&socket_path) {
                Ok(socket) => {
                    info!("Connected to Discord IPC!");

                    let mut client = Self { socket };

                    let handshake = serde_json::json!({
                        "v": 1,
                        "client_id": DISCORD_CLIENT_ID
                    });

                    Self::send(&mut client.socket, 0, &handshake.to_string())?;

                    let _ = Self::receive(&mut client.socket)?;

                    return Ok(client);
                }

                Err(_) => {}
            }
        }

        Err("Could not connect to Discord IPC".into())
    }
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

fn monitor() -> Result<(), Box<dyn std::error::Error>> {
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
            return Ok(component.version);
        }
    }

    Err("Minecraft component not found".into())
}
