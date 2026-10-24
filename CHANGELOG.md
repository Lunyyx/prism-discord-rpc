# Changelog

## 0.3.0

### Added

- Native Windows support (Discord IPC through named pipes, `java.exe` / `javaw.exe` detection)
- Windows installer script (`install-windows.ps1`) with optional start at logon
- Automatic Windows x64 builds
- Automatic Linux arm64 builds
- Automatic macOS arm64 builds
- Elapsed playtime shown in the Discord activity
- macOS documentation

### Changed

- Changed the old `minecraft` variable in the README to plain text
- Added arm64 support to the installation script for both Linux and macOS
- CI now runs tests on Linux, macOS and Windows

### Fixed

- The documented `profile_name` template variable now works

## 0.2.1

### Added

- macOS IPC support
- macOS installation support

**Full Changelog**: https://github.com/Lunyyx/prism-discord-rpc/compare/0.2.0...0.2.1