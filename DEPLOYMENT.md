# Server Deployment Guide

Deploy TERNIMAL forum server on a VPS (DigitalOcean recommended).

## Prerequisites

- Ubuntu 24.04 LTS VPS ($4/month recommended)
- Domain name (optional)
- Basic Linux knowledge

## Quick Setup

### 1. Create VPS

1. Go to [DigitalOcean](https://www.digitalocean.com/products/droplets)
2. Choose Basic plan ($4/month)
3. Select Ubuntu 24.04 LTS
4. Choose any datacenter location
5. Create Droplet

### 2. Initial Server Setup

```bash
# SSH into your new server
ssh root@YOUR_SERVER_IP

# Update system
apt update && apt upgrade -y

# Create user for the forum
useradd -m -s /bin/bash forum
usermod -aG sudo forum
su - forum
```

### 3. Install Dependencies

```bash
# Install PostgreSQL
sudo apt install postgresql postgresql-contrib -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install other dependencies
sudo apt install pkg-config libssl-dev -y
```

### 4. Setup Database

```bash
# Switch to postgres user
sudo -u postgres psql

# In PostgreSQL shell:
CREATE DATABASE forum_db;
CREATE USER forum_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE forum_db TO forum_user;
\q
```

### 5. Deploy Forum Server

```bash
# Clone repository
git clone https://github.com/JRH89/cyber-forum.git
cd cyber-forum/server

# Build server
cargo build --release

# Set environment variables
export DATABASE_URL="postgresql://forum_user:your_secure_password@localhost/forum_db"
export RUST_LOG=info
export HOST=0.0.0.0
export PORT=8080

# Run server
./target/release/forum_server
```

### 6. Setup Systemd Service

```bash
# Create service file
sudo tee /etc/systemd/system/ternimal.service > /dev/null <<EOF
[Unit]
Description=TERNIMAL Forum Server
After=network.target

[Service]
Type=simple
User=forum
WorkingDirectory=/home/forum/cyber-forum/server
Environment="DATABASE_URL=postgresql://forum_user:your_secure_password@localhost/forum_db"
Environment="RUST_LOG=info"
Environment="HOST=0.0.0.0"
Environment="PORT=8080"
ExecStart=/home/forum/cyber-forum/server/target/release/forum_server
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable ternimal
sudo systemctl start ternimal

# Check status
sudo systemctl status ternimal
```

### 7. Configure Firewall

```bash
# Allow SSH, HTTP, HTTPS, and custom SSH port
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 2222/tcp
sudo ufw enable
```

### 8. Update Client Configuration

Edit `src/api.rs` in your local client:
```rust
const BASE_URL: &str = "http://YOUR_SERVER_IP:8080";
```

Rebuild and install the client:
```bash
cargo build --release
sudo cp target/release/ternimal /usr/local/bin/
```

## Access Methods

- **TUI Client**: `ternimal` (from any Arch Linux machine)
- **SSH Access**: `ssh forum@YOUR_SERVER_IP -p 2222`
- **API**: `http://YOUR_SERVER_IP:8080`

## Maintenance

### Update Server
```bash
cd ~/cyber-forum/server
git pull
cargo build --release
sudo systemctl restart ternimal
```

### View Logs
```bash
sudo journalctl -u ternimal -f
```

### Database Backup
```bash
pg_dump forum_db > backup_$(date +%Y%m%d).sql
```

## Troubleshooting

### Service Won't Start
```bash
# Check logs
sudo journalctl -u ternimal -n 50

# Common issues:
# - Database connection error (check DATABASE_URL)
# - Port already in use (check with netstat -tlnp)
```

### Can't Connect via SSH
```bash
# Check if SSH server is running
sudo netstat -tlnp | grep 2222

# Check firewall
sudo ufw status
```

### Database Issues
```bash
# Test connection
psql -h localhost -U forum_user -d forum_db

# Reset database (WARNING: deletes all data)
sudo -u postgres psql -c "DROP DATABASE forum_db; CREATE DATABASE forum_db;"
```
