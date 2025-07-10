# Database TUI Client

A universal Terminal User Interface (TUI) client for database management with Vim-like interactions.

> **Warning:** This project is in the beginning stages of development and has an unstable API and workflow.

## Features

### Currently Supported
- **Redis** - Read-only operations
- **Read-only mode** - Browse and query your databases safely
- **Executing custom query** - You can execute any query of any type (RW-mode)

### Planned Features
- **PostgreSQL support** - Full integration with Postgres databases
- **MySQL support** - Complete MySQL database management
- **Write operations** - Create, update, and delete functionality for all supported databases
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
    - `/` - Search
    - `n` - Go to next search pattern matching
    - `N` - Go to previous search pattern matching
    - `<Enter>` - Get all items in selected object
    - `L|→` - Go to query result widget
    - `<Esc>` - Quit to main page
- Query result widget:
    - `j|k|↑|↓` - Navigate through records
    - `/` - Search
    - `n` - Go to next search pattern matching
    - `N` - Go to previous search pattern matching
    - `q` - Open query input popup
    - `H|←` - Go to database objects widget
    - `g` - Go to the first record (in future will be replaced with `gg`)
    - `G` - Go to the last record
    - `<Esc>` - Quit
- Search popup:
    - `i` - Activate insert mode
    - `<Esc>` - If in insert mode then activate normal mode else - close popup
    - `<Enter>` - In normal mode, apply search pattern. After this use `<Esc>` to close popup
- Query popup:
    - `i` - Activate insert mode
    - `<Esc>` - If in insert mode then activate normal mode else - close popup
    - `<Enter>` - In normal mode, apply query. After this use `<Esc>` to close popup

## Configuration

Create a configuration file any directory:

```toml
[[connections]]
connection_type = "Redis"
name = "local"
connection_string = "redis://localhost:6379"
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

- Built with [tui-realm](https://github.com/veeso/tui-realm) TUI framework
- Inspired by various database GUI clients
