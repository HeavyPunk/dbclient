# Database TUI Client

A universal Terminal User Interface (TUI) client for database management with Vim-like interactions.

> **Warning:** This project is in the beginning stages of development and has an unstable API and workflow.

## Features

### Currently Supported
- **PostgreSQL support** - Partial integration with Postgres databases
- **Operations** - Read, create, update, and delete functionality for PostgreSQL databases

### Planned Features
- **Redis support** - Full integration with Redis databases
- **MySQL support** - Complete MySQL database management
- **UI improvements** - Better UI interactions (for example: horizontal scrolling)
- **UX improvements** - For example: get notification about any error instead of fall to panic

## Installation

```bash
# Clone the repository
git clone https://github.com/HeavyPunk/dbclient.git
cd dbclient

# Build the project
make build

# Or install directly
make install
```

## Usage

### Basic Usage

```bash
dbclient --config-path config.toml
```

### Keyboard Shortcuts

#### Main page:
- `j|k|↑|↓` - Navigate through connections
- `Enter` - Go to query page with selected connection
- `<Esc>` - Quit

#### Query page
- Database objects widget:
    - `j|k|↑|↓` - Navigate through objects
    - `<Space>` - Open object sub-tree
    - `<Enter>` - Get all items in selected object
    - `l|→` - Go to query result widget
    - `<Esc>` - Quit to main page
- Query result widget:
    - `j|k|↑|↓` - Navigate through records
    - `h|←` - Go to database objects widget
    - `gg` - Go to the first record
    - `G` - Go to the last record
    - `a` - Open popup to add record into database object
    - `i` - Open popup to modify record of database object
    - `dd` - Remove record of database object
    - `<Esc>` - Quit
- Popup:
    - `j|k|↑|↓` - Navigate through fields
    - `i` - Activate insert mode for selected field
    - `<Esc>` in insert mode - Activate normal navigation mode through fields
    - `<Esc>` in normal mode - Close popup
    - `<Enter>` in normal mode - Apply operation

## Configuration

Create a configuration file any directory:

```toml
[[connections]]
connection_type = "Postgres"
name = "local"
connection_string = "postgresql://user:password@localhost/postgres?connect_timeout=10"
```

## Requirements

- rustc >= 1.87.0
- cargo >= 1.87.0
- Terminal with UTF-8 support

## Building from Source

```bash
# Build
cargo build --release --target-dir ./build

# Run tests
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [iocraft](https://github.com/ccbrown/iocraft) TUI framework
- Inspired by various database GUI clients
