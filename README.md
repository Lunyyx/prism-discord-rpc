# prism-discord-rpc

A simple tool to automatically display your Minecraft instance and playtime from Prism Launcher on Discord.

![Discord RPC example](docs/discord.png)

# Build

> [!IMPORTANT]
> Discord must be running for the Rich Presence to work.

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

# Configuration

The configuration file is located at:
```
~/.config/prism-discord-rpc/config.toml
```

## `[discord]`
| Option | Description | Example |
|---|---|---|
| `play_text` | Information displayed in the Rich Presence, in the specified order. | `["Minecraft", "MinecraftVersion"]` |
| `play_text_separator` | Separator between each `play_text` value. | `" - "` |

### `play_text` values

| Value | Description |
|---|---|
| `Prism` | Prism Launcher |
| `Minecraft` | Minecraft |
| `MinecraftVersion` | Minecraft version |
| `ProfileName` | Instance/profile name |

### Examples

```toml
[discord]
play_text = ["Minecraft", "MinecraftVersion"]
play_text_separator = " - "
```

This will display:
``Minecraft - 1.21.1``

```toml
[discord]
play_text = ["ProfileName", "Minecraft", "MinecraftVersion"]
play_text_separator = " | "
```

This will display:
``ATM10 | Minecraft | 1.21.1``

