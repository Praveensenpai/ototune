#!/bin/bash

CYAN='\033[0;36m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${PURPLE}🎵 Installing ototune (Minimal Rust TUI MPD Player)...${NC}\n"

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

# Check Prerequisites
echo -e "${BLUE}🔍 Checking dependencies...${NC}"

if command -v mpd >/dev/null 2>&1 || systemctl is-active --quiet mpd 2>/dev/null; then
    echo -e "  ${GREEN}✔ MPD (Music Player Daemon) is available${NC}\n"
else
    echo -e "  ${YELLOW}⚠️  MPD is not detected as running on system.${NC}"
    echo -e "     Make sure mpd daemon is started (e.g. 'systemctl --user start mpd').\n"
fi

REPO="Praveensenpai/ototune"
RELEASE_URL="https://github.com/${REPO}/releases/latest/download/ototune-linux-x86_64.tar.gz"

LOCAL_DIR=""
if [ -n "${BASH_SOURCE[0]}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
    LOCAL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)"
fi

if [ -n "$LOCAL_DIR" ] && [ -f "$LOCAL_DIR/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    VERSION=$(grep -m1 '^version' "$LOCAL_DIR/Cargo.toml" | cut -d '"' -f2 2>/dev/null || echo "latest")
    echo -e "${BLUE}📦 Local source detected. Building ototune v${VERSION} with Cargo...${NC}"
    cargo build --release --manifest-path "$LOCAL_DIR/Cargo.toml"
    cp "$LOCAL_DIR/target/release/ototune" "$BIN_DIR/ototune"
    INSTALLED_VER="v${VERSION}"
else
    LATEST_TAG=$(curl -4 -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep -o '"tag_name": "[^"]*"' | cut -d'"' -f4)
    [ -z "$LATEST_TAG" ] && LATEST_TAG="latest"
    echo -e "${BLUE}📦 Downloading ototune ${LATEST_TAG} pre-compiled binary from GitHub Releases...${NC}"
    TMP_DIR=$(mktemp -d)
    if curl -4 -fL --connect-timeout 10 --retry 3 -sS "$RELEASE_URL" -o "$TMP_DIR/ototune.tar.gz"; then
        tar -xzf "$TMP_DIR/ototune.tar.gz" -C "$TMP_DIR"
        if [ -f "$TMP_DIR/ototune" ]; then
            cp "$TMP_DIR/ototune" "$BIN_DIR/ototune"
        elif [ -f "$TMP_DIR/dist/ototune" ]; then
            cp "$TMP_DIR/dist/ototune" "$BIN_DIR/ototune"
        fi
        rm -rf "$TMP_DIR"
        INSTALLED_VER="${LATEST_TAG}"
    else
        rm -rf "$TMP_DIR"
        echo -e "${RED}❌ Failed to download pre-compiled release.${NC}"
        exit 1
    fi
fi

if [ ! -f "$BIN_DIR/ototune" ] || [ ! -s "$BIN_DIR/ototune" ]; then
    echo -e "${RED}❌ Error: Failed to install ototune binary!${NC}"
    exit 1
fi

chmod +x "$BIN_DIR/ototune"
echo -e "${GREEN}✔ Installed ototune ${INSTALLED_VER} to ${BIN_DIR}/ototune${NC}"

# Shell alias setup
SHELL_CONFIGS=("$HOME/.bashrc" "$HOME/.zshrc")
ALIAS_LINE="alias ototune='$HOME/.local/bin/ototune'"

for config in "${SHELL_CONFIGS[@]}"; do
    if [ -f "$config" ]; then
        if ! grep -q "alias ototune=" "$config" 2>/dev/null; then
            echo "" >> "$config"
            echo "$ALIAS_LINE" >> "$config"
            echo -e "${BLUE}📝 Added ototune alias to $config${NC}"
        fi
    fi
done

echo -e "\n${GREEN}${BOLD}🎉 ototune ${INSTALLED_VER} installation completed!${NC}"
echo -e "Run it anytime with: ${CYAN}ototune${NC}"
