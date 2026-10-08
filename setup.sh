#!/bin/bash
set -eo pipefail

export DEBIAN_FRONTEND=noninteractive

echo "Updating and installing system dependencies..."
sudo apt-get update
sudo apt-get install -y \
    curl \
    wget \
    xz-utils \
    cmake \
    ninja-build \
    pkg-config \
    build-essential \
    git \
    python3 \
    jq \
    unzip \
    libssl-dev \
    libxcb-xfixes0-dev \
    libxcb-shape0-dev \
    libxcb-xkb-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev

# Dynamically retrieve Rust version from rust-toolchain.toml, fallback to stable
RUST_VERSION=$(grep 'channel' rust-toolchain.toml 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "stable")

echo "Installing Rust ($RUST_VERSION)..."
export RUSTUP_HOME="$HOME/.rustup"
export CARGO_HOME="$HOME/.cargo"
export PATH="$CARGO_HOME/bin:$PATH"

if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "$RUST_VERSION"
else
    rustup default "$RUST_VERSION"
fi
rustup component add clippy rustfmt

# Dynamically retrieve Zig version from AGENTS.md, fallback to latest stable available
ZIG_VERSION=$(grep -o 'Zig [0-9]\+\.[0-9]\+\.[0-9]\+' AGENTS.md 2>/dev/null | head -n 1 | awk '{print $2}')
if [ -z "$ZIG_VERSION" ]; then
    # Fallback to fetching the latest version from ziglang if not specified
    ZIG_VERSION=$(curl -s https://ziglang.org/download/index.json | jq -r 'keys | .[]' | grep -v 'master' | sort -V | tail -n 1)
fi

echo "Installing Zig ($ZIG_VERSION)..."
if ! command -v zig &> /dev/null || [[ "$(zig version)" != "$ZIG_VERSION" ]]; then
    cd /tmp
    # Use standard format for URL based on how Zig constructs its download links
    ZIG_URL=$(curl -s https://ziglang.org/download/index.json | jq -r '."'"${ZIG_VERSION}"'" | to_entries[] | select(.key | match("x86_64-linux|linux-x86_64")) | .value.tarball')
    if [ -z "$ZIG_URL" ] || [ "$ZIG_URL" == "null" ]; then
        # Default to the most common pattern
        ZIG_URL="https://ziglang.org/download/$ZIG_VERSION/zig-linux-x86_64-$ZIG_VERSION.tar.xz"
        # Alternative pattern to check
        if ! curl --output /dev/null --silent --head --fail "$ZIG_URL"; then
             ZIG_URL="https://ziglang.org/download/$ZIG_VERSION/zig-x86_64-linux-$ZIG_VERSION.tar.xz"
        fi
    fi
    ZIG_TARBALL=$(basename "$ZIG_URL")

    wget -q "$ZIG_URL" || { echo "Failed to download Zig from $ZIG_URL"; exit 1; }

    if [ -f "$ZIG_TARBALL" ]; then
        tar xf "$ZIG_TARBALL"
        ZIG_DIR=${ZIG_TARBALL%.tar.xz}
        sudo rm -rf /usr/local/zig
        sudo mv "$ZIG_DIR" /usr/local/zig
        sudo ln -sf /usr/local/zig/zig /usr/local/bin/zig
        rm "$ZIG_TARBALL"
    fi
    cd -
fi

echo "Installing just..."
if ! command -v just &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | sudo bash -s -- --to /usr/local/bin
fi

echo "Installing cargo-nextest..."
if ! command -v cargo-nextest &> /dev/null; then
    curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C "$CARGO_HOME/bin"
fi

echo "Installing Bun..."
export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
if ! command -v bun &> /dev/null; then
    curl -fsSL https://bun.sh/install | bash
fi

echo "Setting up environment variables for libghostty-vt..."
export LIBGHOSTTY_VT_OPTIMIZE="Debug"
export LIBGHOSTTY_VT_SIMD="true"

# Persist environment variables for interactive testing
if ! grep -q 'LIBGHOSTTY_VT_OPTIMIZE' ~/.bashrc; then
    {
        echo 'export LIBGHOSTTY_VT_OPTIMIZE="Debug"'
        echo 'export LIBGHOSTTY_VT_SIMD="true"'
        echo 'export PATH="$HOME/.cargo/bin:$HOME/.bun/bin:/usr/local/bin:$PATH"'
    } >> ~/.bashrc
fi

echo "Building project to cache dependencies..."
cargo build || { echo "Cargo build failed, trying to ignore it as dependencies should be fetched"; true; }

echo "Setup script completed successfully!"
