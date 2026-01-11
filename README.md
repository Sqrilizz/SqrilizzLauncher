# Sqrilizz Launcher

A modern Minecraft launcher built with Tauri and Vue 3.

## Features

- Mod and modpack management
- Multiple game profiles
- Modrinth integration for content discovery
- Automatic updates
- Support for different Minecraft versions and mod loaders

## System Requirements

- Windows 10+, Linux (Ubuntu 20.04+,Fedora based system),
- 4 GB RAM (8 GB recommended)
- 500 MB free disk space

## Installation

Download the installer for your platform:

- **Windows**: `.exe` file
- **Linux**: `.deb` or `.rpm` file


Run the installer and follow the instructions.

## Building from Source

Requirements:
- Rust 1.90+
- Node.js 20+
- pnpm 9+

```bash
# Install dependencies
pnpm install

# Development mode
pnpm app:dev

# Build for production
pnpm app:build
```

See `docs/BUILDING.md` for detailed build instructions.

## Known Issues

**Linux**: File picker dialogs may crash due to WebKit limitations. Use drag-and-drop instead.

## Documentation

- [Building Guide](docs/BUILDING.md)
- [macOS Build Instructions](docs/BUILD_MACOS.md)
- [AppImage Build Instructions](docs/BUILD_APPIMAGE.md)
- [Changelog](CHANGELOG.md)

## License

GPL-3.0 - see COPYING.md

## Credits

- Based on [Modrinth Launcher](https://modrinth.com/app) code
- Built with [Tauri](https://tauri.app/), [Vue 3](https://vuejs.org/), and [Modrinth API](https://docs.modrinth.com/)
- Special thanks to [ashtray](https://github.com/ashtray01) for beta testing
