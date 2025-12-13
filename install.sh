#!/bin/bash
set -e

# Cyber Forum Installer
# Installs the TUI client, creates a desktop entry, and sets up the hotkey.

APP_NAME="cyber-forum"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_PATH="$HOME/.local/share/icons"
REPO_DIR="$(pwd)"

echo "🚀 Installing Cyber Forum..."

# 0. Check Dependencies & OS
echo "🔍 Checking system..."

# Secret Club Check 🔒
if ! grep -q "Omarchy" /etc/os-release && ! grep -q "Arch Linux" /etc/os-release; then
    echo "⛔ ACCESS DENIED."
    echo "   This forum is exclusively for Omarchy users."
    echo "   Go install a real OS."
    exit 1
fi
echo "✅ Welcome, Omarchy user."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust (cargo) not found."
    echo "   Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust is installed."
fi

if ! command -v cc &> /dev/null; then
    echo "❌ C compiler (cc) not found."
    echo "   Please install 'build-essential' (Ubuntu/Debian) or 'base-devel' (Arch)."
    # We can't easily auto-install system packages cross-distro without sudo/guessing package manager
    # But for Omarchy (Arch), we could try:
    if command -v pacman &> /dev/null; then
         echo "   Attempting to install base-devel..."
         sudo pacman -S --needed base-devel
    else
         exit 1
    fi
else
    echo "✅ C compiler found."
fi

# 1. Build the client
echo "📦 Building release binary..."
cargo build --release --bin cyber-forum

# 2. Install binary
echo "📂 Installing binary to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp target/release/cyber-forum "$BIN_DIR/"

# 3. Create Desktop Entry
echo "🖥️ Creating desktop entry..."
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_PATH"

# (Optional: You could download a cool icon here, using a placeholder for now)
# curl -o "$ICON_PATH/cyber-forum.png" https://example.com/icon.png

cat <<EOF > "$DESKTOP_DIR/$APP_NAME.desktop"
[Desktop Entry]
Type=Application
Name=Cyber Forum
Comment=The exclusive terminal forum for Omarchy
Exec=$BIN_DIR/cyber-forum
Icon=utilities-terminal
Terminal=true
Categories=Network;ConsoleOnly;
Keywords=forum;tui;omarchy;
EOF

# 4. User Setup
echo "👤 User Setup"
echo "   We will create a local config so you don't have to login every time."
read -p "   Enter Username [$USER]: " INPUT_USER
FORUM_USER=${INPUT_USER:-$USER}
read -s -p "   Enter Password: " FORUM_PASS
echo ""

CONFIG_DIR="$HOME/.config/cyber-forum"
mkdir -p "$CONFIG_DIR"
cat <<EOF > "$CONFIG_DIR/config.json"
{
  "username": "$FORUM_USER",
  "password": "$FORUM_PASS"
}
EOF
echo "   ✅ Config saved to $CONFIG_DIR/config.json"

# 5. Hotkey Setup Instructions
echo "✅ Installation complete!"
echo ""
echo "To set up the Super+Shift+P hotkey:"
echo "  - GNOME: Settings -> Keyboard -> View and Customize Shortcuts -> Custom Shortcuts"
echo "  - KDE: System Settings -> Shortcuts -> Custom Shortcuts"
echo "  - Hyprland/Sway/i3: Add 'bindsym \$mod+Shift+p exec $BIN_DIR/cyber-forum' to your config."
echo ""
echo "Run 'cyber-forum' to start!"
