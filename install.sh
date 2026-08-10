#!/bin/bash
# Standalone installer for ototune (Praveensenpai/ototune)

CYAN='\033[0;36m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${PURPLE}🎵 Installing ototune (Minimal Rust TUI MPD Player)...${NC}\n"

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# If running inside local git repo directory
if [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    echo -e "${CYAN}Building release binary from source...${NC}"
    cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
    install -m 755 "$SCRIPT_DIR/target/release/ototune" "$BIN_DIR/ototune"
else
    # Fetch from GitHub release or fallback to cargo install
    echo -e "${CYAN}Fetching latest ototune binary...${NC}"
    LATEST_TAG=$(curl -s https://api.github.com/repos/Praveensenpai/ototune/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -n "$LATEST_TAG" ]; then
        DOWNLOAD_URL="https://github.com/Praveensenpai/ototune/releases/download/${LATEST_TAG}/ototune-x86_64-unknown-linux-gnu.tar.gz"
        TMP_DIR=$(mktemp -d)
        if curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$TMP_DIR" 2>/dev/null; then
            install -m 755 "$TMP_DIR/ototune" "$BIN_DIR/ototune"
            rm -rf "$TMP_DIR"
        else
            echo -e "${YELLOW}Release binary unavailable. Compiling via cargo...${NC}"
            cargo install --git https://github.com/Praveensenpai/ototune.git --root "$HOME/.local" || true
        fi
    else
        cargo install --git https://github.com/Praveensenpai/ototune.git --root "$HOME/.local" || true
    fi
fi

echo -e "${GREEN}✔ Installed ototune to ~/.local/bin/ototune${NC}"
