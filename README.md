<div align="center">
  # 🎵 ototune ✨

  *Minimal aesthetic Rust TUI MPD player tailored for daily listening and AJATT audio immersion.* 🌸
</div>

---

## 🍡 ⁺ . ⊹ Features . ⁺ ☁️

- 🎧 **Context-Aware Immersion**: Press `i` to instantly shuffle & loop the active folder (e.g. `current` or `immersionpod`), or `Shift+I` for full library immersion.
- 📍 **Playback Position Persistence**: Automatically remembers your exact audio timestamp (`08:42`), last played track, and folder across app reboots (`m` toggle).
- 🔀 **Full Playback Controls**: Toggle Random (`r`), Repeat (`e`), Single track loop (`l`), and Volume (`+`/`-`).
- 🙈 **Smart Hidden File Filtering**: Cleanly filters out `.stfolder` and Syncthing trash items by default (`.` toggle key).
- 🔍 **Live Search Filter**: Press `/` to filter your active queue in real-time.
- 🎨 **Ratatui Aesthetics**: Cute dark mode palette with Japanese font & Unicode character support.

---

## 🍓 ⁺ . ⊹ Quick Start . ⁺ ✨

### 🪄 One-Liner Install

```bash
curl -sSL -H 'Cache-Control: no-cache' "https://raw.githubusercontent.com/Praveensenpai/ototune/main/install.sh?v=$(date +%s)" | bash
```

### ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| **`Tab`** | Toggle focus between Library Browser & Active Queue |
| **`Space`** | Play / Pause |
| **`i`** | Context Immersion Mode (active folder or root) |
| **`I` (Shift+I)** | Global Full Library Immersion Mode |
| **`m`** | Toggle Resume Mode (Remember exact timestamp) |
| **`r`** | Toggle Random playback (`[Random: ON/OFF]`) |
| **`e`** | Toggle Repeat mode (`[Repeat: ON/OFF]`) |
| **`l`** | Toggle Single track loop (`[Single: ON/OFF]`) |
| **`.`** | Toggle showing hidden files (`.stfolder`, etc.) |
| **`+` / `-`** | Volume Up / Down by 5% |
| **`Right` / `Left`** | Seek forward / backward by 5s |
| **`/`** | Live fuzzy queue search |
| **`?`** | Help cheat-sheet modal overlay |
| **`q`** | Quit |

---

<div align="center">
  Made with 💖 by Praveen
</div>
