#!/usr/bin/env bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
PLASMOID_DIR="$HOME/.local/share/plasma/plasmoids/org.kde.plasma.antigravity"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

# Handle --uninstall flag
if [ "$1" = "--uninstall" ] || [ "$1" = "-u" ]; then
    echo "=== Uninstalling Antigravity KDE Plasma 6 Widget ==="
    echo "Stopping and disabling systemd service..."
    systemctl --user stop agy-daemon.service 2>/dev/null || true
    systemctl --user disable agy-daemon.service 2>/dev/null || true
    
    echo "Removing installed files..."
    rm -f "$BIN_DIR/agy-daemon"
    rm -rf "$PLASMOID_DIR"
    rm -f "$SYSTEMD_USER_DIR/agy-daemon.service"
    rm -f "$ICON_DIR/antigravity.svg"
    rm -f "/dev/shm/agy-state.json"
    
    systemctl --user daemon-reload
    echo "✓ Uninstallation complete."
    exit 0
fi

echo "========================================================="
echo "   Antigravity KDE Plasma 6 Widget & Daemon Installer   "
echo "========================================================="

# 1. Determine daemon binary source (Prebuilt vs Build from Source)
DAEMON_SOURCE=""

# Check if prebuilt binary exists in bin/ or dist/
if [ -f "$PROJECT_DIR/bin/agy-daemon" ]; then
    DAEMON_SOURCE="$PROJECT_DIR/bin/agy-daemon"
elif [ -f "$PROJECT_DIR/dist/agy-daemon" ]; then
    DAEMON_SOURCE="$PROJECT_DIR/dist/agy-daemon"
fi

if [ "$1" = "--prebuilt" ] && [ -n "$DAEMON_SOURCE" ]; then
    echo "[1/4] Using prebuilt agy-daemon binary: $DAEMON_SOURCE"
elif command -v cargo >/dev/null 2>&1 && [ -d "$PROJECT_DIR/daemon" ]; then
    echo "[1/4] Building optimized daemon binary from source (Rust native CPU)..."
    cd "$PROJECT_DIR/daemon"
    RUSTFLAGS="-C target-cpu=native" cargo build --release
    DAEMON_SOURCE="$PROJECT_DIR/daemon/target/release/agy-daemon"
    cd "$PROJECT_DIR"
elif [ -n "$DAEMON_SOURCE" ]; then
    echo "[1/4] Cargo not found, falling back to prebuilt binary: $DAEMON_SOURCE"
elif [ -f "$PROJECT_DIR/daemon/target/release/agy-daemon" ]; then
    echo "[1/4] Using existing target/release/agy-daemon binary"
    DAEMON_SOURCE="$PROJECT_DIR/daemon/target/release/agy-daemon"
else
    echo "ERROR: Neither cargo (Rust) nor prebuilt agy-daemon binary found!"
    echo "Please install Rust (cargo) or download a release package with prebuilt binaries."
    exit 1
fi

# Stop running daemon before replacing binary
systemctl --user stop agy-daemon.service 2>/dev/null || true
mkdir -p "$BIN_DIR"
install -m 755 "$DAEMON_SOURCE" "$BIN_DIR/agy-daemon"
echo "  -> Installed binary to: $BIN_DIR/agy-daemon"

# 2. Setup Plasmoid & Icons
echo "[2/4] Installing Plasma 6 Plasmoid and System Icon..."
mkdir -p "$HOME/.local/share/plasma/plasmoids"
rm -rf "$PLASMOID_DIR"
cp -r "$PROJECT_DIR/plasmoid/org.kde.plasma.antigravity" "$PLASMOID_DIR"
echo "  -> Installed plasmoid to: $PLASMOID_DIR"

mkdir -p "$ICON_DIR"
cp "$PROJECT_DIR/plasmoid/org.kde.plasma.antigravity/contents/assets/antigravity.svg" "$ICON_DIR/antigravity.svg"
echo "  -> Installed icon to: $ICON_DIR/antigravity.svg"

# 3. Setup and enable systemd user service
echo "[3/4] Setting up systemd user service..."
mkdir -p "$SYSTEMD_USER_DIR"
cp "$PROJECT_DIR/systemd/agy-daemon.service" "$SYSTEMD_USER_DIR/agy-daemon.service"
systemctl --user daemon-reload
systemctl --user enable --now agy-daemon.service
echo "  -> Service agy-daemon.service enabled and started"

# 4. Verify installation & shared memory
echo "[4/4] Verifying setup..."
sleep 1
if [ -f "/dev/shm/agy-state.json" ]; then
    echo "  -> ✓ OK: Shared memory /dev/shm/agy-state.json is active!"
else
    echo "  -> Notice: Initializing state via one-shot dump..."
    "$BIN_DIR/agy-daemon" dump > /dev/null || true
fi

echo ""
echo "========================================================="
echo "   ✓ Installation Finished Successfully!                "
echo "========================================================="
echo ""
echo "Quick Start:"
echo " 1. Right-click your KDE Plasma Panel and select 'Add Widgets...'"
echo " 2. Search for 'Antigravity Agent Monitor' and drag it to your Panel or System Tray."
echo ""
echo "To preview the widget in a standalone window:"
echo "    plasmawindowed org.kde.plasma.antigravity"
echo ""
echo "Daemon management:"
echo "    systemctl --user status agy-daemon.service"
echo "    agy-daemon dump"
echo ""
