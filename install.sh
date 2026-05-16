#!/bin/bash

echo "🚀 Starting MintClip Installation..."

# 1. Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust and Cargo are not installed."
    echo "Please install Rust first by running: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2. Install X11 Clipboard Dependencies (Requires sudo)
echo "📦 Checking system dependencies (may prompt for sudo password)..."
sudo apt-get update
sudo apt-get install -y xcb libxcb-shape0-dev libxcb-xfixes0-dev

# 3. Kill existing instances to prevent "Text file busy" overwrite errors
echo "🛑 Stopping any running MintClip instances..."
pkill -f mintclip || true

# 4. Build the release binary
echo "🔨 Building optimized release binary (this may take a minute)..."
cargo build --release

# 5. Move binary and make it executable
echo "🚚 Installing binary to ~/.local/bin..."
mkdir -p ~/.local/bin
# We use 'rm -f' first just in case pkill was too slow
rm -f ~/.local/bin/mintclip 
cp target/release/mintclip ~/.local/bin/mintclip
chmod +x ~/.local/bin/mintclip

# 6. Install the application icon
echo "🖼️ Installing application icon..."
mkdir -p ~/.local/share/icons
if [ -f "assets/icon.png" ]; then
    cp assets/icon.png ~/.local/share/icons/mintclip.png
else
    echo "⚠️ Warning: assets/icon.png not found. Skipping icon."
fi

# 7. Create the Application Menu Shortcut
echo "📝 Creating application menu entry..."
mkdir -p ~/.local/share/applications
cat <<EOF > ~/.local/share/applications/mintclip.desktop
[Desktop Entry]
Name=MintClip
Comment=Native Linux Clipboard Manager
Exec=$HOME/.local/bin/mintclip
Icon=$HOME/.local/share/icons/mintclip.png
Terminal=false
Type=Application
Categories=Utility;
EOF

# 8. Set up the background daemon to start on boot
echo "⚙️ Configuring background daemon autostart..."
mkdir -p ~/.config/autostart
cat <<EOF > ~/.config/autostart/mintclip-daemon.desktop
[Desktop Entry]
Type=Application
Exec=sh -c "sleep 3 && $HOME/.local/bin/mintclip --daemon"
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Name[en_US]=MintClip Daemon
Name=MintClip Daemon
Comment=Background clipboard listener for MintClip
Icon=$HOME/.local/share/icons/mintclip.png
EOF

# 9. Start the new daemon right now so the user doesn't have to reboot!
echo "🔄 Starting the MintClip background daemon..."
nohup $HOME/.local/bin/mintclip --daemon > /dev/null 2>&1 &

echo "======================================"
echo "✅ Installation Complete!"
echo "MintClip is now running in the background and available in your App Menu."
echo "To finish setup, open your Keyboard Shortcuts and bind 'Super + V' to: $HOME/.local/bin/mintclip"
