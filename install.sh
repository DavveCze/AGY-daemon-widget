#!/usr/bin/env bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
PLASMOID_DIR="$HOME/.local/share/plasma/plasmoids/org.kde.plasma.antigravity"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
CONFIG_DIR="$HOME/.config/agenticom"
CONFIG_FILE="$CONFIG_DIR/config.json"

# Parse CLI flags
UNINSTALL=false
PREBUILT=false
ENABLE_ALL=false
SELECTED_MODULES=""

for arg in "$@"; do
    case "$arg" in
        --uninstall|-u)
            UNINSTALL=true
            ;;
        --prebuilt)
            PREBUILT=true
            ;;
        --all)
            ENABLE_ALL=true
            ;;
        --modules=*)
            SELECTED_MODULES="${arg#*=}"
            ;;
        *)
            ;;
    esac
done

# Handle --uninstall flag
if [ "$UNINSTALL" = true ]; then
    echo "=== Uninstalling AgentiCom: AI CLI Monitoring System ==="
    echo "Stopping and disabling systemd service..."
    systemctl --user stop agenticom-daemon.service agy-daemon.service 2>/dev/null || true
    systemctl --user disable agenticom-daemon.service agy-daemon.service 2>/dev/null || true
    
    echo "Removing installed files..."
    rm -f "$BIN_DIR/agenticom-daemon" "$BIN_DIR/agy-daemon"
    rm -rf "$PLASMOID_DIR"
    rm -f "$SYSTEMD_USER_DIR/agenticom-daemon.service" "$SYSTEMD_USER_DIR/agy-daemon.service"
    rm -f "$ICON_DIR/antigravity.svg"
    rm -f "/dev/shm/agenticom-state.json" "/dev/shm/agenticom-state.json.tmp"
    rm -f "/dev/shm/agy-state.json" "/dev/shm/agy-state.json.tmp"
    
    systemctl --user daemon-reload
    echo "✓ Uninstallation complete."
    exit 0
fi

echo "========================================================="
echo "   AgentiCom: AI CLI Monitoring System Installer        "
echo "========================================================="

# 1. Module Selection & Configuration
ENABLE_AGY=true
ENABLE_CLAUDE=true
ENABLE_CODEX=false

if [ "$ENABLE_ALL" = true ]; then
    ENABLE_AGY=true
    ENABLE_CLAUDE=true
    ENABLE_CODEX=true
elif [ -n "$SELECTED_MODULES" ]; then
    ENABLE_AGY=false
    ENABLE_CLAUDE=false
    ENABLE_CODEX=false
    IFS=',' read -ra MODS <<< "$SELECTED_MODULES"
    for m in "${MODS[@]}"; do
        case "$m" in
            agy|antigravity) ENABLE_AGY=true ;;
            claude)          ENABLE_CLAUDE=true ;;
            codex)           ENABLE_CODEX=true ;;
            all)             ENABLE_AGY=true; ENABLE_CLAUDE=true; ENABLE_CODEX=true ;;
        esac
    done
fi

echo "[1/5] Configuring Provider Modules:"
echo "  - Antigravity (Gemini): $([ "$ENABLE_AGY" = true ] && echo "✓ ENABLED" || echo "✗ DISABLED")"
echo "  - Claude Code (Anthropic): $([ "$ENABLE_CLAUDE" = true ] && echo "✓ ENABLED" || echo "✗ DISABLED")"
echo "  - OpenAI Codex CLI: $([ "$ENABLE_CODEX" = true ] && echo "✓ ENABLED" || echo "✗ DISABLED")"

# System check for binaries
echo "  Detected CLI binaries in PATH:"
command -v agy >/dev/null 2>&1 && echo "    • agy: $(command -v agy)" || echo "    • agy: Not found in PATH (will monitor if installed later)"
command -v claude >/dev/null 2>&1 && echo "    • claude: $(command -v claude)" || echo "    • claude: Not found in PATH (will monitor if installed later)"
command -v codex >/dev/null 2>&1 && echo "    • codex: $(command -v codex)" || echo "    • codex: Not found in PATH (will monitor if installed later)"

mkdir -p "$CONFIG_DIR"
cat <<EOF > "$CONFIG_FILE"
{
  "poll_interval_ms": 1000,
  "notifications_enabled": true,
  "notification_sound": true,
  "providers": {
    "antigravity": {
      "enabled": $ENABLE_AGY
    },
    "claude": {
      "enabled": $ENABLE_CLAUDE
    },
    "codex": {
      "enabled": $ENABLE_CODEX
    }
  }
}
EOF
echo "  -> Saved configuration to: $CONFIG_FILE"

# 2. Determine daemon binary source
DAEMON_SOURCE=""
if [ -f "$PROJECT_DIR/bin/agy-daemon" ]; then
    DAEMON_SOURCE="$PROJECT_DIR/bin/agy-daemon"
elif [ -f "$PROJECT_DIR/dist/agy-daemon" ]; then
    DAEMON_SOURCE="$PROJECT_DIR/dist/agy-daemon"
fi

if [ "$PREBUILT" = true ] && [ -n "$DAEMON_SOURCE" ]; then
    echo "[2/5] Using prebuilt daemon binary: $DAEMON_SOURCE"
elif command -v cargo >/dev/null 2>&1 && [ -d "$PROJECT_DIR/daemon" ]; then
    echo "[2/5] Building optimized AgentiCom daemon from source (Rust native CPU)..."
    cd "$PROJECT_DIR/daemon"
    RUSTFLAGS="-C target-cpu=native" cargo build --release
    DAEMON_SOURCE="$PROJECT_DIR/daemon/target/release/agy-daemon"
    cd "$PROJECT_DIR"
elif [ -n "$DAEMON_SOURCE" ]; then
    echo "[2/5] Cargo not found, falling back to prebuilt binary: $DAEMON_SOURCE"
elif [ -f "$PROJECT_DIR/daemon/target/release/agy-daemon" ]; then
    echo "[2/5] Using existing target/release binary"
    DAEMON_SOURCE="$PROJECT_DIR/daemon/target/release/agy-daemon"
else
    echo "ERROR: Neither cargo (Rust) nor prebuilt binary found!"
    exit 1
fi

# Stop running daemon before replacing binary
systemctl --user stop agenticom-daemon.service agy-daemon.service 2>/dev/null || true
mkdir -p "$BIN_DIR"
install -m 755 "$DAEMON_SOURCE" "$BIN_DIR/agenticom-daemon"
ln -sf "$BIN_DIR/agenticom-daemon" "$BIN_DIR/agy-daemon"
echo "  -> Installed binary to: $BIN_DIR/agenticom-daemon"
echo "  -> Created compatibility symlink: $BIN_DIR/agy-daemon"

# 3. Setup Plasmoid & Icons
echo "[3/5] Installing Plasma 6 Plasmoid and System Icon..."
mkdir -p "$HOME/.local/share/plasma/plasmoids"
rm -rf "$PLASMOID_DIR"
cp -r "$PROJECT_DIR/plasmoid/org.kde.plasma.antigravity" "$PLASMOID_DIR"
echo "  -> Installed plasmoid to: $PLASMOID_DIR"

mkdir -p "$ICON_DIR"
cp "$PROJECT_DIR/plasmoid/org.kde.plasma.antigravity/contents/assets/antigravity.svg" "$ICON_DIR/antigravity.svg"
echo "  -> Installed icon to: $ICON_DIR/antigravity.svg"

# 4. Setup and enable systemd user service
echo "[4/5] Setting up systemd user service..."
mkdir -p "$SYSTEMD_USER_DIR"
cp "$PROJECT_DIR/systemd/agenticom-daemon.service" "$SYSTEMD_USER_DIR/agenticom-daemon.service"
# Remove old service if present
rm -f "$SYSTEMD_USER_DIR/agy-daemon.service"
systemctl --user daemon-reload
systemctl --user enable --now agenticom-daemon.service
echo "  -> Service agenticom-daemon.service enabled and started"

# 5. Verify installation & shared memory
echo "[5/5] Verifying setup..."
sleep 1
if [ -f "/dev/shm/agenticom-state.json" ]; then
    echo "  -> ✓ OK: Shared memory /dev/shm/agenticom-state.json is active!"
elif [ -f "/dev/shm/agy-state.json" ]; then
    echo "  -> ✓ OK: Shared memory /dev/shm/agy-state.json is active!"
else
    echo "  -> Notice: Initializing state via one-shot dump..."
    "$BIN_DIR/agenticom-daemon" dump > /dev/null || true
fi

echo ""
echo "========================================================="
echo "   ✓ AgentiCom Installation Finished Successfully!       "
echo "========================================================="
echo ""
echo "Quick Start:"
echo " 1. Right-click your KDE Plasma Panel and select 'Add Widgets...'"
echo " 2. Search for 'AgentiCom: AI CLI Monitor' and drag it to your Panel."
echo ""
echo "To preview the widget in a standalone window:"
echo "    plasmawindowed org.kde.plasma.antigravity"
echo ""
echo "Daemon management:"
echo "    systemctl --user status agenticom-daemon.service"
echo "    agenticom-daemon dump"
echo ""
