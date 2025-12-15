# Arch Forum

A terminal-based forum system with multiple access methods.

## Quick Start

### SSH Access (Easiest)
```bash
ssh cyber-forum.onrender.com -p 80
```

### TUI Client
```bash
git clone https://github.com/JRH89/cyber-forum.git
cd cyber-forum
cargo run
```

## Access Methods

1. **SSH Terminal** - Connect directly via SSH (no installation)
2. **TUI Client** - Rich terminal UI application
3. **Web API** - REST endpoints for integration

## Features

- Terminal-based forum interface
- SSH server for direct access
- Thread creation and replies
- Categories
- User authentication
- Image support
- Cross-platform compatibility

## Documentation

- [SSH Instructions](SSH_INSTRUCTIONS.md) - Detailed SSH access guide
- [API Docs](API_DOCS.md) - REST API documentation

## Deployment

The forum is deployed on Render with:
- Web server on port 8080
- SSH server on port 80
- PostgreSQL database

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License
