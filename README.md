# 🗑️ toss (toss-rs)

> **A blazing-fast, FreeDesktop.org-compliant Rust alternative to `trash-cli` with a beautiful TUI.**

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Ratatui](https://img.shields.io/badge/TUI-Ratatui-green.svg)](https://ratatui.rs)

<p align="center">
  <img src="assets/demo.gif" alt="toss Demo" width="850">
</p>

`toss` is a high-performance terminal utility that manages trashed files in compliance with the FreeDesktop.org Trash Specification. It safely records original paths, deletion timestamps, and file sizes in `~/.local/share/Trash`.

---

## ⚡ Key Features

- **⚡ Blazing Fast**: Written in pure Rust for instant execution and minimal resource usage.
- **🖥️ Beautiful Interactive TUI**: Terminal user interface powered by [`ratatui`](https://ratatui.rs) and [`crossterm`](https://crates.io/crates/crossterm).
- **🔍 Live Search & Filtering**: Press `/` in the list view to search trashed files on-the-fly.
- **♻️ Interactive Multi-Select Restore**: Easily toggle files with `Space` and restore them to their original location.
- **🧹 Age-Based Purging**: Clear items older than N days with `toss empty <days>`.
- **🎯 Glob Pattern Matching**: Remove specific items from trash using patterns with `toss rm "*.log"`.
- **🛡️ 100% FreeDesktop Compliant**: Fully compatible with GNOME, KDE, XFCE, and other standard desktop environments.

---

## 🚀 Quick Start

### 🪄 One-Liner Magic (Recommended)

Paste this into your terminal to install `toss` automatically:

```bash
curl -sSL -H "Cache-Control: no-cache" https://raw.githubusercontent.com/Praveensenpai/toss-rs/main/install.sh | bash
```

<br>

### 🛠️ Building From Source

```bash
git clone https://github.com/Praveensenpai/toss-rs.git
cd toss-rs
cargo build --release
install -Dm 755 target/release/toss ~/.local/bin/toss
```

<br>

### ⚡ Manual Shell Autocompletions Setup

If installing manually from source or via `cargo install`, generate and install autocompletions for your shell:

#### **Zsh**
```zsh
mkdir -p ~/.local/share/zsh/site-functions
toss completions zsh > ~/.local/share/zsh/site-functions/_toss
echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc
```

#### **Bash**
```bash
mkdir -p ~/.local/share/bash-completion/completions
toss completions bash > ~/.local/share/bash-completion/completions/toss
```

#### **Fish**
```fish
mkdir -p ~/.config/fish/completions
toss completions fish > ~/.config/fish/completions/toss.fish
```

<br>

### ⚠️ Aliasing `rm` to `toss put` (Optional)

If you want `rm` to safely move files to trash instead of permanently unlinking them, run the one-liner for your shell:

#### **Zsh**
```zsh
echo "alias rm='toss put'" >> ~/.zshrc
```

#### **Bash**
```bash
echo "alias rm='toss put'" >> ~/.bashrc
```

#### **Fish**
```fish
echo "alias rm='toss put'" >> ~/.config/fish/config.fish
```

> **Note**: To bypass the alias and use standard `rm`, invoke `\rm` or `/bin/rm`.

---

## 📖 Usage

### Commands

| Command | Description |
| :--- | :--- |
| `toss put <files>` | Move files or directories to the trashcan |
| `toss list` / `toss` | Open interactive TUI to browse trashed files |
| `toss restore` | Open interactive multi-select TUI to restore files |
| `toss restore --overwrite` | Restore files and overwrite existing files if present |
| `toss empty` | Permanently delete all trashed files |
| `toss empty 7` | Empty files trashed more than 7 days ago |
| `toss rm "*.o"` | Delete trashed items matching a glob pattern |

### TUI Keybindings

- **`j` / `k` or `↓` / `↑`**: Navigate through entries
- **`/`**: Live search & filter
- **`Space` / `Enter`**: Toggle selection
- **`v`**: Visual range mode (auto-selects rows as you navigate with `j`/`k`)
- **`r`**: Restore selected / highlighted items
- **`d`**: Permanently delete selected / highlighted items
- **`a`**: Select all / Deselect all
- **`q` or `Esc`**: Quit / Close

---

## 🛠️ Architecture & Tech Stack

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Language** | Rust 2021 | Native speed & safety |
| **TUI Engine** | [`ratatui`](https://ratatui.rs) & `crossterm` | Terminal layout, tables, and event handling |
| **CLI Parser** | [`clap`](https://crates.io/crates/clap) | Subcommands and flags |
| **Trash Spec** | `std::fs` & `chrono` | `.trashinfo` metadata generation & parsing |

---

## 📜 License

Distributed under the [MIT License](LICENSE).
