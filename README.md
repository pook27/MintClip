# MintClip 📋

A lightning-fast, natively-rendered clipboard manager for Linux, built entirely in Rust. 

MintClip is designed to bring the fluid, power-user experience of the Windows `Win+V` clipboard to Linux desktops. It runs as a lightweight background daemon to monitor your clipboard, and provides a beautiful, keyboard-driven UI to search, preview, and manage your history.

## ✨ Features

* **Instant UI:** Built with `egui` for a blazing-fast, immediate-mode graphical interface.
* **Smart Content Detection:** Automatically categorizes copied items (URLs, Emails, JSON, Paths, Code, Tokens).
* **Developer First:** Full syntax highlighting for code snippets using `syntect`.
* **Fuzzy Search:** Instantly find old snippets with typo-tolerant fuzzy matching via `skim`.
* **Image Support:** Visual previews for copied images with automatic hard-drive garbage collection.
* **Bidirectional Text:** Flawless rendering for Right-to-Left languages (Hebrew, Arabic) using `unicode-bidi`.
* **Persistent History:** Pin your most used items (API keys, bash commands) so they never disappear.

---

## 🚀 Full Installation Guide

Follow these steps to build MintClip from source and integrate it perfectly into your Linux desktop environment.

### Step 1: Install Dependencies
Make sure you have the standard Rust toolchain installed. You will also need the required Linux clipboard dependencies for X11:
```bash
sudo apt update
sudo apt install xcb libxcb-shape0-dev libxcb-xfixes0-dev

```

### Step 2: Clone & Build from Source

Download the repository and compile the highly-optimized release binary.

```bash
https://github.com/pook27/MintClip.git
git clone https://github.com/pook27/MintClip.git
cd mintclip
cargo build --release

```

*Note: Your compiled executable is now located at `/path/to/mintclip/target/release/mintclip`.*

### Step 3: Run the Background Daemon on Boot

MintClip needs to run silently in the background to record your clipboard history. To ensure it starts automatically when you turn on your computer:

1. Open your Linux application menu and search for **Startup Applications**.
2. Click the **`+`** button and select **Custom command**.
3. Fill in the details:
* **Name:** `MintClip Daemon`
* **Command:** `sh -c "sleep 3 && /path/to/mintclip/target/release/mintclip --daemon"`
* *(Note: Replace `/path/to/mintclip` with the actual absolute path to your folder. The `sleep 3` ensures your system clipboard is fully loaded before MintClip attaches to it).*


4. Click **Save**.
5. *To start it immediately without rebooting, run the command above in your terminal.*

### Step 4: Bind the `Win + V` Shortcut

MintClip acts as a toggle: running the executable once opens the UI, running it again closes it instantly.

1. Open your Linux application menu and search for **Keyboard**.
2. Navigate to the **Shortcuts** tab and select **Custom Shortcuts** on the left.
3. Click the **Add custom shortcut** button at the bottom.
* **Name:** `MintClip UI`
* **Command:** `/path/to/mintclip/target/release/mintclip`


4. Click **Add**.
5. Find the new "MintClip UI" entry in your list, double-click the unassigned keyboard binding at the bottom, and press **Super + V** (the Windows Key + V).

**🎉 You're done! Copy some text and press `Super + V` to see MintClip in action.**

---

## 🛠️ Built With

* [egui](https://github.com/emilk/egui) - GUI framework
* [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard access
* [syntect](https://github.com/trishume/syntect) - Syntax highlighting
* [fuzzy-matcher](https://github.com/lotabout/fuzzy-matcher) - Search algorithms

## 📜 License

This project is open-source and available under the MIT License.
