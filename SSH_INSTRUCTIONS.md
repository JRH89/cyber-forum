# Arch Forum - SSH Access Instructions

## Quick Connect
```bash
ssh cyber-forum.onrender.com -p 80
```

## What is Arch Forum?
Arch Forum is a terminal-based forum system accessible via SSH. No installation required - just SSH in!

## SSH Access Methods

### Method 1: Direct SSH (Recommended)
```bash
# Connect directly
ssh cyber-forum.onrender.com -p 80

# Or with username (any username works)
ssh user@cyber-forum.onrender.com -p 80
```

### Method 2: Using SSH Config
Add to your `~/.ssh/config`:
```
Host archforum
    HostName cyber-forum.onrender.com
    Port 80
    User user
```

Then simply:
```bash
ssh archforum
```

### Method 3: Using telnet (if SSH blocked)
```bash
telnet cyber-forum.onrender.com 80
```

## First Time Connection
1. You'll see: "Arch Forum SSH Server"
2. Type anything containing "arch" or "linux" for verification
3. Welcome message appears
4. You're in!

## Available Commands
- `list` - Show recent threads
- `post` - Create new thread
- `reply` - Reply to a thread
- `help` - Show help
- `quit` - Exit

## Example Session
```
$ ssh cyber-forum.onrender.com -p 80
Arch Forum SSH Server
Arch Linux verification required...
arch
Arch Linux verified! Welcome.

=== ARCH FORUM ===
Commands: list, post, reply, help, quit
forum> list
Recent threads:
[1] [SOLVED] pacman database lock
[2] [HELP] AUR package signing
[3] [DISCUSS] systemd vs openrc
forum> quit
Connection closed.
```

## Platform-Specific Instructions

### Windows
**Using PowerShell:**
```powershell
ssh cyber-forum.onrender.com -p 80
```

**Using PuTTY:**
1. Hostname: `cyber-forum.onrender.com`
2. Port: `80`
3. Connection type: SSH
4. Click Open

### macOS/Linux
```bash
ssh cyber-forum.onrender.com -p 80
```

### Android
1. Install Termius or ConnectBot
2. Host: `cyber-forum.onrender.com`
3. Port: `80`
4. Connect

### iOS
1. Install Termius or Blink Shell
2. Host: `cyber-forum.onrender.com`
3. Port: `80`
4. Connect

## Tips
- Any username works - just type anything for verification
- The forum is fully terminal-based - no graphics
- All posts are stored in a shared database
- Works from any computer with SSH access
- Perfect for library computers, school labs, or work terminals

## Troubleshooting
- If port 80 is blocked, try port 443
- If SSH is blocked, try telnet
- No password required - just verify with "arch" or "linux"

## Alternative: TUI Client
For a richer experience, install the TUI client:
```bash
git clone https://github.com/JRH89/cyber-forum.git
cd cyber-forum
cargo run
```

Enjoy the terminal forum experience!
