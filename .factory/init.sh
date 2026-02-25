#!/bin/bash
set -e

# Install Rust toolchain if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Ensure cargo is on PATH (idempotent)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Install system dependencies (Amazon Linux 2023 / Fedora-like)
if command -v dnf &> /dev/null; then
    echo "Installing system dependencies via dnf..."
    sudo dnf install -y \
        libxdo-devel \
        libX11-devel \
        libxcb-devel \
        libXtst-devel \
        xorg-x11-server-Xvfb \
        gcc \
        pkg-config \
        2>/dev/null || echo "Some packages may not be available, continuing..."
fi

# Install ffmpeg if not present
if ! command -v ffmpeg &> /dev/null; then
    echo "Installing ffmpeg..."
    sudo dnf install -y ffmpeg 2>/dev/null || {
        echo "ffmpeg not in default repos, trying alternatives..."
        # Try Amazon Linux extras or manual install
        sudo dnf install -y https://dl.fedoraproject.org/pub/epel/epel-release-latest-2023.noarch.rpm 2>/dev/null || true
        sudo dnf install -y ffmpeg 2>/dev/null || echo "WARNING: ffmpeg installation failed. Screen recording tests will be skipped."
    }
fi

# Install gh CLI if not present (needed for CI milestone)
if ! command -v gh &> /dev/null; then
    echo "Installing GitHub CLI..."
    sudo dnf install -y 'dnf-command(config-manager)' 2>/dev/null || true
    sudo dnf config-manager --add-repo https://cli.github.com/packages/rpm/gh-cli.repo 2>/dev/null || true
    sudo dnf install -y gh 2>/dev/null || echo "WARNING: gh CLI installation failed. GitHub operations may need manual setup."
fi

# Initialize Cargo project if not exists
cd /home/ec2-user/code/work/xctrl
if [ ! -f Cargo.toml ]; then
    echo "Cargo.toml will be created by the first worker feature."
fi

# Initialize git repo if not exists
if [ ! -d .git ]; then
    git init
    echo "target/" > .gitignore
    echo ".factory/validation/" >> .gitignore
    git add -A
    git commit -m "Initial repository setup" --allow-empty || true
fi

# Fetch dependencies if Cargo.toml exists
if [ -f Cargo.toml ]; then
    cargo fetch 2>/dev/null || true
fi

echo "Environment setup complete."
