# prism-discord-rpc

A simple tool to automatically display your Minecraft instance and playtime from Prism Launcher on Discord.

![Discord RPC example](docs/discord.png)

# Installation

> [!IMPORTANT]
> Discord must be running for the Rich Presence to work.

## Automatic installation

This script currently supports Linux x86_64 / arm64 and macOS x86_64 (Intel) / arm64 (Apple Silicon).

You can run the following command to install this tool:

```bash
curl -fsSL https://raw.githubusercontent.com/Lunyyx/prism-discord-rpc/refs/heads/master/install.sh | bash
```

## Service management

### Linux

Check status:
```bash
systemctl --user status prism-discord-rpc
```

Start service:
```bash
systemctl --user start prism-discord-rpc
```

Stop service:
```bash
systemctl --user stop prism-discord-rpc
```

Restart service:
```bash
systemctl --user restart prism-discord-rpc
```

View logs:
```bash
journalctl --user -u prism-discord-rpc
```

### macOS

Check status:
```bash
launchctl print gui/$(id -u)/prism-discord-rpc | grep -E "state =|pid ="
```

Start service:
```bash
launchctl start prism-discord-rpc
```

Stop service:
```bash
launchctl stop prism-discord-rpc
```

Restart service:
```bash
launchctl stop prism-discord-rpc
launchctl start prism-discord-rpc
```

View logs:
```bash
tail -f ~/Library/Logs/prism-discord-rpc.error.log
```

# Configuration

The configuration file is located at:
```
~/.config/prism-discord-rpc/config.toml
```

> [!WARNING]
> You are responsible for the text displayed through this tool. Using offensive, illegal, or otherwise prohibited text may result in action being taken against your Discord account.

## `[discord_activity]`
| Option | Description | Example |
|---|---|---|
| `name` | Name displayed in the Rich Presence. Supports variables. | `"Minecraft"` |
| `details` | Details displayed in the Rich Presence. Supports variables. | `"Playing {{ minecraft_version }}"` |
| `state` | State displayed in the Rich Presence. Supports variables. | `"{{ profile_name }}"` |

### Available variables

| Variable | Description |
|---|---|
| `{{ minecraft_version }}` | Minecraft version |
| `{{ profile_name }}` | Instance/profile name |

### Examples

```toml
[discord_activity]
name = "Minecraft"
details = "{{ minecraft_version }}"
state = "{{ profile_name }}"
```

This will display:

```
Minecraft
1.21.1
ATM10
```

You can freely combine and order variables:
```toml
[discord_activity]
name = "Minecraft - {{ minecraft_version }}"
details = "Playing {{ profile_name }}"
state = "Prism Launcher"
```

This will display:
```
Minecraft - 1.21.1
Playing ATM10
Prism Launcher
```

# Build

1. Clone the repository
```bash
git clone https://github.com/Lunyyx/prism-discord-rpc.git
```

2. Open the project directory
```bash
cd prism-discord-rpc
```

3. Build the project
```bash
cargo build --release
```

4. Execute the project
```bash
./target/release/prism-discord-rpc
```
