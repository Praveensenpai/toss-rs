#!/usr/bin/env bash
set -e

REPO="Praveensenpai/toss-rs"
BINARY_NAME="toss"
INSTALL_DIR="$HOME/.local/bin"

echo "🗑️  Installing toss (toss-rs)..."

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ]; then
    echo "❌ Currently only Linux is supported by this installer."
    exit 1
fi

if [ "$ARCH" != "x86_64" ]; then
    echo "❌ Currently only x86_64 architecture is supported."
    exit 1
fi

TAG=$(curl -4 -sSL --connect-timeout 10 --retry 3 "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$TAG" ]; then
    echo "⚠️  No official release tag found. Building locally via Cargo..."
    if command -v cargo >/dev/null 2>&1; then
        cargo build --release
        mkdir -p "$INSTALL_DIR"
        install -m 755 target/release/toss "$INSTALL_DIR/toss"
        echo "✔ Installed toss to $INSTALL_DIR/toss!"
        exit 0
    else
        echo "❌ Cargo is not installed. Please install Rust/Cargo."
        exit 1
    fi
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG/toss-x86_64-unknown-linux-gnu.tar.gz"

mkdir -p "$INSTALL_DIR"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "📥 Downloading toss $TAG..."
curl -4 -sSL --connect-timeout 10 --retry 3 "$DOWNLOAD_URL" | tar -xz -C "$TMP_DIR"

install -m 755 "$TMP_DIR/toss" "$INSTALL_DIR/toss"

echo "✔ Successfully installed toss to $INSTALL_DIR/toss!"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Note: $INSTALL_DIR is not in your PATH."
    echo "   Add it to your shell config (~/.bashrc or ~/.zshrc):"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
