use log::info;
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use crate::models::Session;
use crate::config;
use crate::template::format_text;

const DISCORD_CLIENT_ID: &str = "1538515152788258837";

pub struct DiscordClient {
    socket: UnixStream,
}

impl DiscordClient {
    pub fn update_from_session(&mut self, session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        let config = config::CONFIG.get().unwrap();

        let activity_name = format_text(&config.discord_activity.name, session)?;
        let activity_details = format_text(&config.discord_activity.details, session)?;
        let activity_state = format_text(&config.discord_activity.state, session)?;

        self.set_activity(&activity_name, &activity_details, &activity_state)
    }

    pub fn clear_activity(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn set_activity(
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

    pub fn receive(socket: &mut UnixStream) -> Result<(u32, String), Box<dyn std::error::Error>> {
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

    pub fn send(
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

    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let os = std::env::consts::OS;

        let runtime_dir: Option<String> = if os == "macos" {
            Some(std::env::var("TMPDIR")?)
        } else if os == "linux" {
            Some(std::env::var("XDG_RUNTIME_DIR")?)
        } else {
            None
        };

        if let Some(runtime_dir) = runtime_dir {
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
        }

        Err("Could not connect to Discord IPC".into())
    }
}