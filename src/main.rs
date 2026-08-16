use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;
use serde::Deserialize;
use sysinfo::System;
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

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
    path: PathBuf,
    minecraft_version: String,
    java_version: String,
}

struct Session {
    instance: Instance,
    started_at: Instant,
}

struct DiscordClient {
    socket: UnixStream,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    monitor()
}

impl DiscordClient {
    fn clear_activity(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": null
            },
            "nonce": uuid::Uuid::new_v4().to_string()
        });

        Self::send(
            &mut self.socket,
            1,
            &payload.to_string(),
        )?;

        let (opcode, response) = Self::receive(&mut self.socket)?;

        println!("CLEAR_ACTIVITY response opcode: {}", opcode);
        println!("CLEAR_ACTIVITY response: {}", response);

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

        Self::send(
            &mut self.socket,
            1,
            &payload.to_string(),
        )?;

        let (opcode, response) = Self::receive(&mut self.socket)?;

        println!("SET_ACTIVITY response opcode: {}", opcode);
        println!("SET_ACTIVITY response: {}", response);

        Ok(())
    }

    fn receive(
        socket: &mut UnixStream,
    ) -> Result<(u32, String), Box<dyn std::error::Error>> {
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
            let socket_path = format!(
                "{}/discord-ipc-{}",
                runtime_dir,
                i
            );

            match UnixStream::connect(&socket_path) {
                Ok(socket) => {
                    println!("Connected to Discord IPC!");
                    println!("Socket: {}", socket_path);

                    let mut client = Self { socket };

                    let handshake = serde_json::json!({
                        "v": 1,
                        "client_id": DISCORD_CLIENT_ID
                    });

                    Self::send(
                        &mut client.socket,
                        0,
                        &handshake.to_string(),
                    )?;

                    println!("Handshake sent!");

                    let (opcode, payload) = Self::receive(&mut client.socket)?;

                    println!("Handshake response opcode: {}", opcode);
                    println!("Handshake response: {}", payload);

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

fn update_session(session: &mut Option<Session>, instance: Option<Instance>) -> SessionEvent{
        match instance {
            Some(instance) => {
                if session.is_none() {
                    println!("Minecraft started !");
                    println!("Instance : {}", instance.name);
                    println!("Minecraft : {}", instance.minecraft_version);
                    println!("Java : {}", instance.java_version);
                    println!("Path : {}", instance.path.display());

                    *session = Some(Session { 
                        instance, 
                        started_at: Instant::now() 
                    });

                    return SessionEvent::Started;
                }

                if let Some(session) = session.as_ref() {
                    let elapsed = session.started_at.elapsed();

                    println!("Playtime: {} secondes", elapsed.as_secs());
                }

                SessionEvent::None
            }

            None => {
                if let Some(current_session) = session.as_ref() {
                    println!("Minecraft stopped!");
                    println!(
                        "Session: {} secondes",
                        current_session.started_at.elapsed().as_secs()
                    );
                    
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

    let mut discord = DiscordClient::connect()?;

    loop {
        system.refresh_all();

        let instance = find_instance(&system);

        let event = update_session(&mut session, instance);

        match event {
            SessionEvent::Started => {
                if let Some(session) = &session {
                    let state = format!(
                        "Minecraft {}",
                        session.instance.minecraft_version
                    );

                    discord.set_activity(
                        "Minecraft",
                        &session.instance.name,
                        &state,
                    )?;

                    println!("Discord activity updated!");
                }
            }

            SessionEvent::Stopped => {
                discord.clear_activity()?;

                println!("Discord activity cleared!");
            }

            SessionEvent::None => {}
        }

        thread::sleep(Duration::from_secs(2));
    }
}

fn find_instance(system: &System) -> Option<Instance> {
    for(_pid, process) in system.processes() {
        let name = process.name().to_string_lossy();
        let command = process.cmd();

        if name == "java" && command.iter().any(|arg| arg == "org.prismlauncher.EntryPoint") {
            for arg in command {
                let arg = arg.to_string_lossy();

                if let Some(path) = arg.strip_prefix("-Djava.library.path=") {
                    if let Some(instance_path) = Path::new(path).parent() {
                        match read_instance(instance_path) {
                            Ok(instance) => {
                                return Some(instance);
                            }
                            Err(error) => {
                                println!("Erreur : {}", error);
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
    let mut java_version = String::from("Unknown");

    for line in config.lines() {
        if let Some(value) = line.strip_prefix("name=") {
            name = value.to_string();
        }

        if let Some(value) = line.strip_prefix("JavaVersion=") {
            java_version = value.to_string();
        }
    }

    let minecraft_version = read_minecraft_version(path)?;

    Ok(Instance {
        name,
        path: path.to_path_buf(),
        minecraft_version,
        java_version,
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