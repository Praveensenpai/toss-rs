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

TAG=$(curl -4 -sSL -H "Cache-Control: no-cache" -H "Pragma: no-cache" --connect-timeout 10 --retry 3 "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$TAG" ]; then
    echo "⚠️  No official release tag found. Building locally via Cargo..."
    if command -v cargo >/dev/null 2>&1; then
        cargo build --release
        mkdir -p "$INSTALL_DIR"
        install -m 755 target/release/toss "$INSTALL_DIR/toss"
        echo "✔ Installed toss to $INSTALL_DIR/toss!"
    else
        echo "❌ Cargo is not installed. Please install Rust/Cargo."
        exit 1
    fi
else
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG/toss-x86_64-unknown-linux-gnu.tar.gz"
    mkdir -p "$INSTALL_DIR"
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    echo "📥 Downloading toss $TAG..."
    curl -4 -sSL -H "Cache-Control: no-cache" -H "Pragma: no-cache" --connect-timeout 10 --retry 3 "$DOWNLOAD_URL" | tar -xz -C "$TMP_DIR"
    install -m 755 "$TMP_DIR/toss" "$INSTALL_DIR/toss"
    echo "✔ Successfully installed toss to $INSTALL_DIR/toss!"
fi

# Detect active login shell from $SHELL
USER_SHELL="$(basename "${SHELL:-zsh}")"

# Automatic Shell Completion Installation
echo "⚡ Deploying automatic shell completions..."

# Zsh completions
ZSH_COMP_DIR="$HOME/.local/share/zsh/site-functions"
mkdir -p "$ZSH_COMP_DIR"
"$INSTALL_DIR/toss" completions zsh > "$ZSH_COMP_DIR/_toss"
if [ -f "$HOME/.zshrc" ] && ! grep -q "zsh/site-functions" "$HOME/.zshrc"; then
    echo -e "\nfpath=(\$HOME/.local/share/zsh/site-functions \$fpath)" >> "$HOME/.zshrc"
fi

# Bash completions
BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
mkdir -p "$BASH_COMP_DIR"
"$INSTALL_DIR/toss" completions bash > "$BASH_COMP_DIR/toss"

# Fish completions
FISH_COMP_DIR="$HOME/.config/fish/completions"
mkdir -p "$FISH_COMP_DIR"
"$INSTALL_DIR/toss" completions fish > "$FISH_COMP_DIR/toss.fish"

echo "✔ Autocompletions installed for Zsh, Bash, and Fish!"

# Determine config file for active shell
CONFIG_FILE=""
case "$USER_SHELL" in
    zsh)  CONFIG_FILE="$HOME/.zshrc" ;;
    bash) CONFIG_FILE="$HOME/.bashrc" ;;
    fish) CONFIG_FILE="$HOME/.config/fish/config.fish" ;;
esac

# Interactive Alias Setup
if [ -t 0 ] || [ -c /dev/tty ]; then
    TTY_DEV="/dev/tty"
    echo ""
    if [ -n "$CONFIG_FILE" ] && [ -f "$CONFIG_FILE" ]; then
        EXISTING_ALIAS=$(grep -E "^\s*alias rm=" "$CONFIG_FILE" || true)
        if [ -n "$EXISTING_ALIAS" ]; then
            echo "⚠️  Existing alias found in $CONFIG_FILE:"
            echo "   $EXISTING_ALIAS"
            read -r -p "❓ Do you want to overwrite it with 'alias rm=\"toss put\"'? (y/N): " OVERWRITE_REPLY < "$TTY_DEV" || OVERWRITE_REPLY="n"
            if [[ "$OVERWRITE_REPLY" =~ ^[Yy]$ ]]; then
                sed -i '/^\s*alias rm=/d' "$CONFIG_FILE"
                echo "alias rm='toss put'" >> "$CONFIG_FILE"
                echo "✔ Updated 'alias rm=\"toss put\"' in $CONFIG_FILE"
            else
                echo "ℹ️  Kept existing alias."
            fi
        else
            read -r -p "❓ Do you want to alias 'rm' to 'toss put' in $USER_SHELL? (y/N): " ALIAS_REPLY < "$TTY_DEV" || ALIAS_REPLY="n"
            if [[ "$ALIAS_REPLY" =~ ^[Yy]$ ]]; then
                echo "alias rm='toss put'" >> "$CONFIG_FILE"
                echo "✔ Added 'alias rm=\"toss put\"' to $CONFIG_FILE"
            fi
        fi
    fi
fi

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Note: $INSTALL_DIR is not in your PATH."
    echo "   Add it to your shell config (~/.bashrc or ~/.zshrc):"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
