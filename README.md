# 🎵 ototune

> **Minimalist, Blazing-Fast Aesthetic Rust TUI MPD Client Tailored for Daily Listening & AJATT Audio Immersion.**

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Ratatui](https://img.shields.io/badge/TUI-ratatui.rs-pink.svg?style=flat-square)](https://ratatui.rs/)
[![MPD](https://img.shields.io/badge/Protocol-MPD-brightgreen.svg?style=flat-square)](https://www.musicpd.org/)

---

```text
┌───────────────────────────────────────────────────────────────┐
│        🎵  O T O T U N E  ──  Minimal Rust MPD TUI            │
└───────────────────────────────────────────────────────────────┘
```

**`ototune`** is a high-performance, keyboard-driven terminal MPD (Music Player Daemon) client written in Rust. Designed to replace heavy legacy C++ tools like `ncmpcpp`, `ototune` provides a modern **Ratatui** dual-pane interface, context-aware AJATT audio immersion shortcuts, automatic playback position persistence, smart hidden file filtering, and zero-latency local MPD socket control.

---

## ✨ Features

- 🎧 **Context-Aware AJATT Immersion**:
  - Press **`i`** inside any anime folder (e.g. `current` or `immersionpod`) to clear the queue, load, shuffle & loop **only that active folder**.
  - Press **`Shift+I` (`I`)** for full library immersion across your entire collection (`/`).
- 📍 **Automatic Playback Position Persistence**:
  - Automatically saves your exact audio timestamp (`08:42`), last played track, active folder, and preference toggles to `~/.config/ototune/state.json`.
  - Resumes exact playback position across app reboots (`m` toggle).
- 🔀 **Dedicated Playback Controls**:
  - Independent keybindings for Random (`r`), Repeat (`e`), Single track loop (`l`), Volume (`+`/`-`), and Seeking (`Right`/`Left`).
- 🙈 **Smart Hidden File Filtering**:
  - Automatically filters out Syncthing metadata (`.stfolder`, `.stfolder.removed-*`) and dotfiles from your library view (`.` toggle key).
- 🔍 **Live Search Filter**:
  - Press `/` to perform instant live fuzzy search filtering across your queue.
- 🎨 **Cute Dark Aesthetics**:
  - Soft pink/cyan color palette with Japanese font rendering and Unicode status indicators.

---

## 📋 Requirements

- **MPD (Music Player Daemon)**: Running locally on `127.0.0.1:6600` (or configured via `MPD_HOST` / `MPD_PORT` environment variables).
- **Terminal**: Modern UTF-8 terminal with Japanese font support (e.g., Alacritty, Kitty, WezTerm, Foot).

---

## 🚀 Installation

### 🪄 One-Liner (Recommended)

Paste this into your terminal to install `ototune` automatically to `~/.local/bin/ototune`:

```bash
curl -sSL -H 'Cache-Control: no-cache' "https://raw.githubusercontent.com/Praveensenpai/ototune/main/install.sh?v=$(date +%s)" | bash
```

<br>

### 📦 Cargo Install

If you have Rust installed via `rustup`:

```bash
cargo install --git https://github.com/Praveensenpai/ototune.git --root ~/.local
```

<br>

### 🛠️ Building & Installing From Source

```bash
git clone https://github.com/Praveensenpai/ototune.git
cd ototune
cargo build --release
install -m 755 target/release/ototune ~/.local/bin/ototune
```

---

## 📖 Usage & Command Line Options

```bash
# Launch ototune (normal player mode)
ototune

# Launch directly into AJATT Immersion Mode (shuffle & loop)
ototune -i

# Launch with hidden files (.stfolder) visible by default
ototune -H

# Launch and automatically resume last saved playback position
ototune -r

# Specify custom MPD address
ototune -a 127.0.0.1:6600
```

---

## ⌨️ Keyboard Shortcuts

| Key | Action | Status Indicator |
| :--- | :--- | :--- |
| **`Tab`** | Toggle focus between Library Browser & Active Queue | — |
| **`Space`** | Play / Pause toggle | `▶ PLAYING` / `⏸ PAUSED` |
| **`i`** | Context Immersion (shuffles active folder or root) | `✨ 🎧 Shuffled Immersion: 'folder'` |
| **`Shift+I` (`I`)** | Global Full Library Immersion (shuffles `/`) | `✨ 🎧 Global Full Library Immersion` |
| **`m`** | Toggle **Resume Mode** (remembers exact timestamp) | `[Resume: ON]` / `[Resume: OFF]` |
| **`r`** | Toggle **Random** playback | `[Random: ON]` / `[Random: OFF]` |
| **`e`** | Toggle **Repeat** mode | `[Repeat: ON]` / `[Repeat: OFF]` |
| **`l`** | Toggle **Single** track loop | `[Single: ON]` / `[Single: OFF]` |
| **`.`** | Toggle showing hidden files (`.stfolder`, etc.) | `📂 Library [path] (All)` |
| **`+` / `-`** | Volume Up / Down by 5% | `🔊 85%` |
| **`Right` / `Left`** | Seek forward / backward by 5 seconds | `⏩ +5s` / `⏪ -5s` |
| **`Enter`** | Play track / Open directory / Queue folder | — |
| **`d` / `Delete`** | Remove selected track from Queue | — |
| **`c`** | Clear Queue | — |
| **`/`** | Start Live Queue Search | `🔍 SEARCH: query█` |
| **`?`** | Help cheat-sheet modal overlay | — |
| **`q`** | Quit | — |

---

## ⚙️ Persistence & Configuration

`ototune` automatically manages state persistence in `~/.config/ototune/state.json`:

```json
{
  "last_folder": "current",
  "last_file": "current/Ore wo Suki nano wa Omae dake ka yo - 01.opus",
  "last_elapsed_secs": 862,
  "resume_mode": true,
  "show_hidden": false
}
```

---

## 🛠️ Architecture & Tech Stack

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Language** | Rust 2021 | Native compilation, ~3-8MB RSS, ~1ms startup |
| **TUI Engine** | [`ratatui`](https://ratatui.rs/) & [`crossterm`](https://crates.io/crates/crossterm) | Custom terminal canvas with dark mode palette |
| **MPD RPC** | [`mpd`](https://crates.io/crates/mpd) & [`tokio`](https://tokio.rs/) | Async TCP client socket communication (`127.0.0.1:6600`) |
| **State Storage** | `serde` & `serde_json` | Local JSON state persistence (`~/.config/ototune/state.json`) |

---

## 📜 License

Distributed under the [MIT License](LICENSE).

<br>

<div align="center">
  Made with 💖 by Praveen
</div>
