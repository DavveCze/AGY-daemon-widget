#!/usr/bin/env bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION="1.0.0"
DIST_DIR="$PROJECT_DIR/dist"
BIN_DIR="$PROJECT_DIR/bin"
PKG_NAME="agenticom-plasma6-widget-v${VERSION}-linux-x86_64"
LEGACY_PKG_NAME="antigravity-plasma6-widget-v${VERSION}-linux-x86_64"
RELEASE_STAGING="$DIST_DIR/$PKG_NAME"

echo "========================================================="
echo "   Building AgentiCom Plasma 6 Release Package (v$VERSION) "
echo "========================================================="

# 1. Clean and prepare directories
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR" "$BIN_DIR" "$RELEASE_STAGING"

# 2. Build Rust daemon
echo "[1/4] Compiling Rust daemon in release mode..."
cd "$PROJECT_DIR/daemon"
cargo build --release
cd "$PROJECT_DIR"

# Copy binary to bin/ and strip debug symbols
cp "$PROJECT_DIR/daemon/target/release/agy-daemon" "$BIN_DIR/agenticom-daemon"
strip --strip-all "$BIN_DIR/agenticom-daemon"
ln -sf "$BIN_DIR/agenticom-daemon" "$BIN_DIR/agy-daemon"
echo "  -> Created stripped binary: $BIN_DIR/agenticom-daemon ($(ls -lh "$BIN_DIR/agenticom-daemon" | awk '{print $5}'))"

# 3. Assemble release staging directory
echo "[2/4] Assembling release files..."
mkdir -p "$RELEASE_STAGING/bin"
mkdir -p "$RELEASE_STAGING/plasmoid"
mkdir -p "$RELEASE_STAGING/systemd"

cp "$BIN_DIR/agenticom-daemon" "$RELEASE_STAGING/bin/agenticom-daemon"
cp "$BIN_DIR/agy-daemon" "$RELEASE_STAGING/bin/agy-daemon"
cp -r "$PROJECT_DIR/plasmoid/org.kde.plasma.antigravity" "$RELEASE_STAGING/plasmoid/"
cp "$PROJECT_DIR/systemd/agenticom-daemon.service" "$RELEASE_STAGING/systemd/"
cp "$PROJECT_DIR/install.sh" "$RELEASE_STAGING/install.sh"
cp "$PROJECT_DIR/README.md" "$RELEASE_STAGING/README.md"
cp "$PROJECT_DIR/LICENSE" "$RELEASE_STAGING/LICENSE"
chmod +x "$RELEASE_STAGING/install.sh" "$RELEASE_STAGING/bin/agenticom-daemon" "$RELEASE_STAGING/bin/agy-daemon"

# 4. Create .plasmoid package (zip format for Plasma package installer)
echo "[3/4] Creating standalone .plasmoid package..."
cd "$PROJECT_DIR/plasmoid/org.kde.plasma.antigravity"
python3 -c "
import zipfile, os
with zipfile.ZipFile('$DIST_DIR/org.kde.plasma.antigravity.plasmoid', 'w', zipfile.ZIP_DEFLATED) as z:
    for root, dirs, files in os.walk('.'):
        for f in files:
            path = os.path.join(root, f)
            z.write(path, arcname=os.path.relpath(path, '.'))
"
echo "  -> Generated: $DIST_DIR/org.kde.plasma.antigravity.plasmoid"

# 5. Create release tarball
echo "[4/4] Creating distribution archive..."
cd "$DIST_DIR"
tar -czf "${PKG_NAME}.tar.gz" "$PKG_NAME"
cp "${PKG_NAME}.tar.gz" "${LEGACY_PKG_NAME}.tar.gz"

# Generate SHA256 checksums
sha256sum "${PKG_NAME}.tar.gz" > "${PKG_NAME}.tar.gz.sha256"
sha256sum "${LEGACY_PKG_NAME}.tar.gz" > "${LEGACY_PKG_NAME}.tar.gz.sha256"
sha256sum "org.kde.plasma.antigravity.plasmoid" > "org.kde.plasma.antigravity.plasmoid.sha256"

echo ""
echo "========================================================="
echo "   ✓ Release Build Successful!                           "
echo "========================================================="
echo "Distribution artifacts generated in $DIST_DIR:"
ls -lh "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.plasmoid
echo ""
echo "Checksums:"
cat "$DIST_DIR"/*.sha256
echo ""
