# Pack Preferences

A Rust GUI application for replicating EVE Online settings across characters. Linux only.

## Features

- **Auto-detection**: Automatically detects running EVE Online instances and their Wine prefixes
- **Character discovery**: Finds all character and user settings files
- **Name resolution**: Resolves character IDs to names via EVE ESI API
- **Settings sync**: Copy settings from one character to others
- **Backup management**: Create and restore backups before making changes
- **Dry-run mode**: Preview changes before applying them

## Installation

### From Release

Download the latest binary from the [Releases](../../releases) page.

### From Source

```bash
cargo build --release
```

## Usage

1. Launch the application
2. If EVE is running, it will auto-detect the Wine prefix
3. Otherwise, click "Browse" to select your Wine prefix (`drive_c` directory)
4. Select a source character to copy settings FROM
5. Select target characters to copy settings TO
6. Enable "Dry Run Mode" to preview changes (recommended first time)
7. Click "Sync Settings"

## Settings Location

EVE Online settings are stored at:
```
{wine_prefix}/users/steamuser/AppData/Local/CCP/EVE/*/settings_Default/
```

Files:
- `core_char_*.dat` - Character-specific settings
- `core_user_*.dat` - User/account settings

## Configuration

App configuration is stored at `~/.config/pack-preferences/config.toml`:
- Last used Wine prefix path
- Window position
- Character name cache

## Building

Requirements:
- Rust 1.70+
- GTK3 development libraries (for rfd file dialogs)

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install libgtk-3-dev

# Build
cargo build --release
```

## License

MIT
