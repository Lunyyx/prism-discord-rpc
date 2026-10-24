use crate::{config, models::Session, template::format_text};
use log::info;
#[cfg(windows)]
use std::fs::File as IpcStream;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream as IpcStream;

const DISCORD_CLIENT_ID: &str = "1538515152788258837";

pub struct DiscordClient {
    socket: IpcStream,
}

impl DiscordClient {
    pub fn update_from_session(
        &mut self,
        session: &Session,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = config::CONFIG.get().unwrap();
        let name = format_text(&config.discord_activity.name, session)?;
        let details = format_text(&config.discord_activity.details, session)?;
        let state = format_text(&config.discord_activity.state, session)?;
        self.set_activity(serde_json::json!({
            "name": name, "details": details, "state": state,
            "timestamps": { "start": session.started_at }
        }))?;
        info!("Discord accepted activity: {} / {}", details, state);
        Ok(())
    }

    pub fn clear_activity(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_activity(serde_json::Value::Null)
    }

    fn set_activity(
        &mut self,
        activity: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": { "pid": std::process::id(), "activity": activity },
            "nonce": uuid::Uuid::new_v4().to_string()
        });
        Self::send(&mut self.socket, 1, &payload.to_string())?;
        Self::checked_response(&mut self.socket)
    }

    fn receive(socket: &mut impl Read) -> Result<(u32, String), Box<dyn std::error::Error>> {
        let mut header = [0u8; 8];
        socket.read_exact(&mut header)?;
        let opcode = u32::from_le_bytes(header[..4].try_into()?);
        let length = u32::from_le_bytes(header[4..].try_into()?) as usize;
        if length > 1024 * 1024 {
            return Err("Discord IPC frame exceeds 1 MiB".into());
        }
        let mut payload = vec![0; length];
        socket.read_exact(&mut payload)?;
        Ok((opcode, String::from_utf8(payload)?))
    }

    fn send(
        socket: &mut impl Write,
        opcode: u32,
        payload: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        socket.write_all(&opcode.to_le_bytes())?;
        socket.write_all(&(payload.len() as u32).to_le_bytes())?;
        socket.write_all(payload.as_bytes())?;
        Ok(())
    }

    fn checked_response(socket: &mut impl Read) -> Result<(), Box<dyn std::error::Error>> {
        let (opcode, payload) = Self::receive(socket)?;
        let response: serde_json::Value = serde_json::from_str(&payload)?;
        if opcode != 1 || response["evt"] == "ERROR" {
            return Err(format!("Discord rejected request: {}", response).into());
        }
        Ok(())
    }

    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(windows)]
        let paths: Vec<String> = (0..10)
            .map(|i| format!(r"\\.\pipe\discord-ipc-{}", i))
            .collect();
        #[cfg(unix)]
        let paths: Vec<String> = {
            let runtime = std::env::var(if cfg!(target_os = "macos") {
                "TMPDIR"
            } else {
                "XDG_RUNTIME_DIR"
            })?;
            (0..10)
                .map(|i| format!("{}/discord-ipc-{}", runtime, i))
                .collect()
        };
        for path in paths {
            #[cfg(windows)]
            let socket = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path);
            #[cfg(unix)]
            let socket = IpcStream::connect(&path);
            if let Ok(socket) = socket {
                let mut client = Self { socket };
                let handshake = serde_json::json!({"v": 1, "client_id": DISCORD_CLIENT_ID});
                Self::send(&mut client.socket, 0, &handshake.to_string())?;
                Self::checked_response(&mut client.socket)?;
                info!("Connected to Discord IPC!");
                return Ok(client);
            }
        }
        Err("Could not connect to Discord IPC".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn frame(opcode: u32, payload: &str) -> Cursor<Vec<u8>> {
        let mut bytes = Vec::new();
        DiscordClient::send(&mut bytes, opcode, payload).unwrap();
        Cursor::new(bytes)
    }
    #[test]
    fn framing_uses_little_endian_byte_lengths() {
        let mut bytes = frame(1, "\u{4e16}\u{754c}");
        assert_eq!(&bytes.get_ref()[..8], &[1, 0, 0, 0, 6, 0, 0, 0]);
        assert_eq!(
            DiscordClient::receive(&mut bytes).unwrap(),
            (1, "\u{4e16}\u{754c}".into())
        );
    }
    #[test]
    fn rejects_errors_close_and_truncated_frames() {
        assert!(
            DiscordClient::checked_response(&mut frame(
                1,
                r#"{"evt":"ERROR","data":{"message":"invalid activity"}}"#
            ))
            .is_err()
        );
        assert!(DiscordClient::checked_response(&mut frame(2, r#"{"message":"closed"}"#)).is_err());
        let mut truncated = frame(1, "{}").into_inner();
        truncated.pop();
        assert!(DiscordClient::receive(&mut Cursor::new(truncated)).is_err());
    }
    #[test]
    fn accepts_ready_and_activity_acknowledgements() {
        assert!(DiscordClient::checked_response(&mut frame(1, r#"{"evt":"READY"}"#)).is_ok());
        assert!(
            DiscordClient::checked_response(&mut frame(1, r#"{"cmd":"SET_ACTIVITY","evt":null}"#))
                .is_ok()
        );
    }
}
