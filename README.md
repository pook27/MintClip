# MintClip 📋

A lightning-fast, natively-rendered clipboard manager for Linux, built entirely in Rust. 

MintClip is designed to bring the fluid, power-user experience of the Windows `Win+V` clipboard to Linux desktops. It runs as a lightweight background daemon to monitor your clipboard, and provides a beautiful, keyboard-driven UI to search, preview, and manage your history.

## ✨ Features

* **Instant UI:** Built with `egui` for a blazing-fast, immediate-mode graphical interface.
* **Smart Content Detection:** Automatically categorizes copied items (URLs, Emails, JSON, Paths, Code).
* **Developer First:** Full syntax highlighting for code snippets using `syntect`.
* **Fuzzy Search:** Instantly find old snippets with typo-tolerant fuzzy matching via `skim`.
* **Image Support:** Visual previews for copied images with automatic hard-drive garbage collection.
* **Bidirectional Text:** Flawless rendering for Right-to-Left languages (Hebrew, Arabic) using `unicode-bidi`.
* **Persistent History:** Pin your most used items (API keys, bash commands) so they never disappear.

## 🚀 Installation

### Prerequisites
Make sure you have the standard Rust toolchain installed, along with the required Linux clipboard dependencies:
```bash
sudo apt update
sudo apt install xcb libxcb-shape0-dev libxcb-xfixes0-dev # For X11 clipboard access

```

### Build from Source

```bash
git clone [https://github.com/YOUR_USERNAME/mintclip.git](https://github.com/YOUR_USERNAME/mintclip.git)
cd mintclip
cargo build --release

```

Your optimized executable will be located at `target/release/mintclip`.

## ⚙️ Configuration & Usage

MintClip operates in two modes: the background daemon (which listens to your clipboard) and the UI (which lets you interact with it).

### 1. Start the Daemon

To ensure MintClip is always recording your history, add the daemon to your Linux Startup Applications:

* **Command:** `sh -c "sleep 3 && /path/to/mintclip/target/release/mintclip --daemon"`
* *(The `sleep 3` ensures the system clipboard manager is fully initialized before MintClip attaches to it).*

### 2. Set Up the UI Shortcut

To open the UI, you just run the executable without the daemon flag. It acts as a toggle: running it once opens the UI, running it again closes it.

* Open your Linux Keyboard Shortcuts settings.
* Add a custom shortcut pointing to `/path/to/mintclip/target/release/mintclip`.
* Bind it to `Super + V` (or your preferred shortcut).

## 🛠️ Built With

* [egui](https://github.com/emilk/egui) - GUI framework
* [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard access
* [syntect](https://github.com/trishume/syntect) - Syntax highlighting
* [fuzzy-matcher](https://github.com/lotabout/fuzzy-matcher) - Search algorithms

## 📜 License

This project is open-source and available under the MIT License.

```

```
