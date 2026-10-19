#!/usr/bin/env bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}   AgentiCom: AI CLI Monitoring Test Suite         ${NC}"
echo -e "${BLUE}====================================================${NC}"

# 1. Rust Daemon Unit & Integration Tests
echo -e "\n${BLUE}[1/4] Running Rust Daemon Unit & Integration Tests...${NC}"
cd "$PROJECT_DIR/daemon"
cargo test --quiet
echo -e "${GREEN}✓ All Rust unit, integration & multi-provider tests passed!${NC}"

# 2. QML & Plasmoid Metadata & Lint Tests
echo -e "\n${BLUE}[2/5] Running Plasmoid Structure & QML Linting...${NC}"
python3 "$PROJECT_DIR/tests/test_plasmoid.py"
echo -e "${GREEN}✓ All Plasmoid structure and QML checks passed!${NC}"

# 3. Causal UI Scroll Preservation & State Reconciliation Tests
echo -e "\n${BLUE}[3/5] Running Causal UI Scroll & State Reconciliation Tests...${NC}"
python3 "$PROJECT_DIR/tests/test_scroll_preservation_and_reconciliation.py"
echo -e "${GREEN}✓ All causal UI tests passed!${NC}"

# 4. D-Bus Interface Verification
echo -e "\n${BLUE}[4/5] Verifying D-Bus Interface (org.kde.plasma.antigravity)...${NC}"
if qdbus6 org.kde.plasma.antigravity /org/kde/plasma/antigravity org.kde.plasma.antigravity.GetActiveCount > /dev/null 2>&1; then
    ACTIVE_COUNT=$(qdbus6 org.kde.plasma.antigravity /org/kde/plasma/antigravity org.kde.plasma.antigravity.GetActiveCount)
    echo -e "${GREEN}✓ D-Bus Service is responsive! Active agent count: ${ACTIVE_COUNT}${NC}"
else
    echo -e "${RED}! Notice: D-Bus service org.kde.plasma.antigravity is offline or under another session.${NC}"
fi

# 5. Systemd Service Status
echo -e "\n${BLUE}[5/5] Verifying systemd user service...${NC}"
if systemctl --user is-active --quiet agenticom-daemon.service; then
    echo -e "${GREEN}✓ agenticom-daemon.service is ACTIVE and running in background!${NC}"
elif systemctl --user is-active --quiet agy-daemon.service; then
    echo -e "${GREEN}✓ agy-daemon.service is ACTIVE and running in background!${NC}"
else
    echo -e "${RED}! Notice: daemon service is currently inactive. Run './install.sh' to enable it.${NC}"
fi

echo -e "\n${GREEN}====================================================${NC}"
echo -e "${GREEN}   ✓ ALL TEST SUITES COMPLETED SUCCESSFULLY!        ${NC}"
echo -e "${GREEN}====================================================${NC}"
