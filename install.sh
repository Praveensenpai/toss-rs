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

# Check for existing rm alias across all user shell config files
FOUND_ALIASES=""
TARGET_FILES=()

for CFG in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.config/fish/config.fish"; do
    if [ -f "$CFG" ]; then
        TARGET_FILES+=("$CFG")
        LINE=$(grep -E "^\s*alias rm=" "$CFG" || true)
        if [ -n "$LINE" ]; then
            FOUND_ALIASES="${FOUND_ALIASES}   $CFG -> $LINE\n"
        fi
    fi
done

# Interactive Alias Setup
if [ -t 0 ] || [ -c /dev/tty ]; then
    TTY_DEV="/dev/tty"
    echo ""
    if [ -n "$FOUND_ALIASES" ]; then
        echo "⚠️  Existing 'rm' alias found in your shell config(s):"
        echo -e "$FOUND_ALIASES"
        read -r -p "❓ Do you want to overwrite with 'alias rm=\"toss put\"'? (y/N): " OVERWRITE_REPLY < "$TTY_DEV" || OVERWRITE_REPLY="n"
        if [[ "$OVERWRITE_REPLY" =~ ^[Yy]$ ]]; then
            for CFG in "${TARGET_FILES[@]}"; do
                sed -i '/^\s*alias rm=/d' "$CFG"
                echo "alias rm='toss put'" >> "$CFG"
                echo "✔ Updated 'alias rm=\"toss put\"' in $CFG"
            done
        else
            echo "ℹ️  Kept existing alias."
        fi
    else
        read -r -p "❓ Do you want to alias 'rm' to 'toss put' in your shell config(s)? (y/N): " ALIAS_REPLY < "$TTY_DEV" || ALIAS_REPLY="n"
        if [[ "$ALIAS_REPLY" =~ ^[Yy]$ ]]; then
            for CFG in "${TARGET_FILES[@]}"; do
                echo "alias rm='toss put'" >> "$CFG"
                echo "✔ Added 'alias rm=\"toss put\"' to $CFG"
            done
        fi
    fi
fi

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Note: $INSTALL_DIR is not in your PATH."
    echo "   Add it to your shell config (~/.bashrc or ~/.zshrc):"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
