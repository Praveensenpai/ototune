<div align="center">
  # 🎵 ototune ✨

  *Minimal aesthetic Rust TUI MPD player tailored for daily listening and AJATT audio immersion.* 🌸

  [![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Ratatui](https://img.shields.io/badge/TUI-ratatui.rs-pink.svg)](https://ratatui.rs/)
  [![MPD](https://img.shields.io/badge/Protocol-MPD-brightgreen.svg)](https://www.musicpd.org/)

</div>

---

## 🍡 ⁺ . ⊹ Overview . ⁺ ☁️

`ototune` is a blazing-fast, lightweight, keyboard-driven terminal MPD (Music Player Daemon) client written in Rust. Designed to replace legacy C++ `ncmpcpp`, `ototune` provides a modern **Ratatui** dual-pane interface, context-aware AJATT audio immersion shortcuts, automatic playback position persistence, and zero-latency local MPD socket control.

---

## ✨ ⁺ . ⊹ Key Features . ⁺ 🌸

- 🎧 **Context-Aware AJATT Immersion**:
  - Press **`i`** inside any anime folder (e.g. `current` or `immersionpod`) to clear the queue and loop & shuffle **only that active folder**.
  - Press **`Shift+I` (`I`)** for full library immersion across your entire collection (`/`).
- 📍 **Automatic Playback Position Persistence**:
  - Automatically saves your exact audio timestamp (`08:42`), last played track, active folder, and preference toggles to `~/.config/ototune/state.json`.
  - Resumes exact playback position across app reboots (`m` toggle).
- 🔀 **Full Playback Controls**:
  - Dedicated keybinds for Random (`r`), Repeat (`e`), Single track loop (`l`), Volume (`+`/`-`), and Seeking (`Right`/`Left`).
- 🙈 **Smart Hidden File Filtering**:
  - Automatically filters out Syncthing metadata (`.stfolder`, `.stfolder.removed-*`) and dotfiles from your library view (`.` toggle key).
- 🔍 **Live Search Filter**:
  - Press `/` to perform instant live fuzzy search filtering across your queue.
- 🎨 **Cute Dark Aesthetics**:
  - Pastel pink/cyan color palette with Japanese font rendering and Unicode status indicators.

---

## 🚀 ⁺ . ⊹ Quick Start . ⁺ 🍓

### 🪄 One-Liner Magic (Recommended)

Paste this into your terminal to install `ototune` automatically:

```bash
curl -sSL -H 'Cache-Control: no-cache' "https://raw.githubusercontent.com/Praveensenpai/ototune/main/install.sh?v=$(date +%s)" | bash
```

<br>

### 🛠️ Building From Source

```bash
git clone https://github.com/Praveensenpai/ototune.git
cd ototune
cargo build --release
install -m 755 target/release/ototune ~/.local/bin/ototune
```

---

## ⌨️ ⁺ . ⊹ Keyboard Shortcuts . ⁺ 📟

| Key | Action | Header Indicator |
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

## 🛠️ ⁺ . ⊹ Tech Stack . ⁺ ✨

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Language** | Rust 2021 | Native compilation, ~3-8MB RSS, ~1ms startup |
| **TUI Canvas** | [`ratatui`](https://ratatui.rs/) & [`crossterm`](https://crates.io/crates/crossterm) | Custom terminal canvas with dark mode palette |
| **MPD RPC** | [`mpd`](https://crates.io/crates/mpd) & [`tokio`](https://tokio.rs/) | Async TCP client socket communication (`127.0.0.1:6600`) |
| **State Storage** | `serde` & `serde_json` | Local JSON state persistence (`~/.config/ototune/state.json`) |

---

<div align="center">
  Made with 💖 by Praveen
</div>
