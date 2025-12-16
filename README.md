# TERNIMAL - Terminal Forum Client

A terminal-based forum system for Arch Linux enthusiasts.

## Quick Start

### Arch Linux Installation (Recommended)
```bash
# Download and install from GitHub Releases
wget https://github.com/JRH89/cyber-forum/releases/latest/download/ternimal-arch.tar.gz
tar -xzf ternimal-arch.tar.gz
./install.sh
```

After installation, simply run:
```bash
ternimal
```

### Manual Installation
```bash
git clone https://github.com/JRH89/cyber-forum.git
cd cyber-forum
cargo build --release
sudo cp target/release/ternimal /usr/local/bin/
```

## Features

- Terminal-based forum interface
- Thread creation and replies
- User authentication and registration
- Categories
- Cross-platform compatibility
- Arch Linux optimized

## TUI Controls

### Navigation
- **↑/↓ Arrow Keys** - Navigate threads/posts
- **← Arrow** - Back to thread list
- **→ Arrow** - Enter thread conversation
- **Enter** - Select item or open thread
- **Esc** - Cancel/exit current mode

### Login Screen
- **Tab** - Switch between username/password fields
- **Enter** - Login
- **Esc** - Quit application

### Forum Actions
- **n** - Create new thread
- **r** - Reply to current thread
- **Tab** - Switch between input fields (when creating)
- **Esc** - Cancel new thread/reply

## Authentication

1. **New Users**: Enter any username and password to register
2. **Existing Users**: Login with your credentials
3. **Password Security**: Passwords are hashed with SHA256

## Server Setup

For self-hosting, deploy on a VPS with:
- Ubuntu 24.04 LTS
- PostgreSQL database
- Rust environment
- Custom ports (SSH: 2222, Web: 8080)

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed server setup instructions.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License
