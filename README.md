# prism-discord-rpc

A simple tool to automatically display your Minecraft instance and playtime from Prism Launcher on Discord.

![Discord RPC example](docs/discord.png)

# Installation

> [!IMPORTANT]
> Discord must be running for the Rich Presence to work.

## Linux / macOS installation

You can run the following command to install this tool:

```bash
curl -fsSL https://raw.githubusercontent.com/Lunyyx/prism-discord-rpc/refs/heads/master/install.sh | bash
```

## Windows

Use the Discord desktop app and launch Minecraft through Prism Launcher. Browser-only Discord cannot receive local Rich Presence.

Build from source with [Rust](https://rustup.rs/) and the Visual Studio C++ build tools (Desktop development with C++, including the Windows SDK):

```powershell
git clone https://github.com/Lunyyx/prism-discord-rpc.git
cd prism-discord-rpc
cargo build --release --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-windows.ps1 -StartAtLogon
```

The installer runs without administrator privileges, starts the helper hidden, and installs it in `%LOCALAPPDATA%\Programs\PrismDiscordRPC`. Omit `-StartAtLogon` to skip adding a Windows sign-in shortcut; an existing shortcut is preserved on reinstall. The execution-policy override applies only to this invocation.

The release workflow also builds `prism-rpc-v<VERSION>-windows-x64.exe`. Once a release includes that asset, it can be installed without Rust using `-BinaryPath`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-windows.ps1 -BinaryPath .\prism-rpc-v<VERSION>-windows-x64.exe -StartAtLogon
```

Replace `<VERSION>` with the downloaded release version. Keep `install-windows.ps1` from the same release/source revision.

### Windows management

```powershell
# Start (also used by the sign-in shortcut; avoids duplicate helper processes)
& "$env:LOCALAPPDATA\Programs\PrismDiscordRPC\Start.ps1"

# Stop
Get-Process prism-discord-rpc -ErrorAction SilentlyContinue | Stop-Process

# View logs (reset each time the helper starts)
Get-Content "$env:LOCALAPPDATA\Programs\PrismDiscordRPC\rpc.log" -Tail 30
```

To disable automatic startup, remove `Prism Discord RPC.lnk` from the folder opened by `shell:startup`. To uninstall, stop the helper, remove that shortcut, and delete `%LOCALAPPDATA%\Programs\PrismDiscordRPC`. Configuration is kept separately and can be removed from `%APPDATA%\prism-discord-rpc`.

The helper retries if Discord is not open yet. If logs say Discord accepted the activity but it is not visible, check Discord's activity-sharing settings. The elapsed timer starts when the helper first detects the game and resets if the helper restarts.

## Linux service management

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

# Configuration

The configuration file is created on first launch:

| Platform | Location |
|---|---|
| Windows | `%APPDATA%\prism-discord-rpc\config.toml` |
| Linux | `$XDG_CONFIG_HOME/prism-discord-rpc/config.toml`, or `~/.config/prism-discord-rpc/config.toml` |
| macOS | `~/Library/Application Support/prism-discord-rpc/config.toml` |

Restart the helper after editing the configuration.

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
| `{{ instance_name }}` | Alias for `profile_name` (used by the default config) |

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
```
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
