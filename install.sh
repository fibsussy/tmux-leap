#!/bin/bash

set -euo pipefail

# Create a temporary directory that will self-destruct
TMP_DIR=$(mktemp -d -t tmux-leap-install.XXXXXX)
START_DIR=$(pwd)
trap 'cd "$START_DIR" && rm -rf "$TMP_DIR"' EXIT INT TERM

# Download PKGBUILD
curl -fsSL -o "$TMP_DIR/PKGBUILD" "https://raw.githubusercontent.com/fibsussy/tmux-leap/main/PKGBUILD"

# Verify we're on Arch Linux
if [ ! -f /etc/arch-release ]; then
    echo "This script only supports Arch Linux. Get your shit together and use the right distro."
    exit 1
fi

cd "$TMP_DIR"
makepkg -si --noconfirm
echo "tmux-leap installed successfully via pacman"
