# MintClip

A lightning-fast, natively-rendered clipboard manager for Linux, built entirely in Rust. 

MintClip is designed to bring a fluid, power-user clipboard experience to Linux desktops. It runs as a lightweight background daemon to monitor your clipboard and provides a beautiful, keyboard-driven UI to search, preview, and manage your history.

## Features

* **Instant UI:** Built with `egui` for a blazing-fast, immediate-mode graphical interface.
* **Auto-Paste:** Clicking or hitting enter on an item instantly closes the UI and pastes the snippet directly into your active window.
* **In-App Customization:** Click the ⚙️ icon to tweak your accent colors, window dimensions, font sizes, and history limits on the fly.
* **Smart Content Detection:** Automatically categorizes copied items (URLs, Emails, JSON, Paths, Code, Tokens).
* **Developer First:** Full syntax highlighting for code snippets using `syntect`.
* **Fuzzy Search:** Instantly find old snippets with typo-tolerant fuzzy matching via `skim`.
* **Image Support:** Visual previews for copied images with automatic hard-drive garbage collection.
* **Bidirectional Text:** Flawless rendering for Right-to-Left languages (Hebrew, Arabic) using `unicode-bidi`.
* **Persistent History:** Pin your most used items (API keys, bash commands) so they never disappear.

---

## Installation

MintClip comes with a one-click installation script that handles dependencies, compilation, desktop entries, and background daemon configuration for Debian/Ubuntu-based systems.


Clone the repository and run the install script:
```bash
git clone https://github.com/pook27/MintClip.git
cd MintClip
chmod +x install.sh
./install.sh

```

---

## Usage & Setup

MintClip acts as a toggle switch: running the executable once opens the UI, and running it again closes it instantly.

To get the best experience, you should map it to **Super + V** (the Windows Key + V) in your Linux settings:

1. Open your Linux application menu and search for **Keyboard** or **Keyboard Shortcuts**.
2. Navigate to **Custom Shortcuts** and click **Add**.
3. Fill in the details:
* **Name:** `MintClip`
* **Command:** `/home/YOUR_USERNAME/.local/bin/mintclip` *(Should be the final output in the installation script)*


4. Assign the shortcut to **Super + V**.

**You're done! Copy some text and press `Super + V` to see MintClip in action.**

---

## Configuration

MintClip creates a dynamic configuration file located at `~/.config/mintclip/config.toml`.

While you can edit this file manually, it is much easier to use the **Settings Modal** built directly into the MintClip UI (accessible via the Gear icon in the top right corner). Your history and cached images are safely stored in the same directory.

---

## Built With

* [egui](https://github.com/emilk/egui) - GUI framework
* [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard access
* [enigo](https://github.com/enigo-rs/enigo) - Keystroke simulation for auto-pasting
* [syntect](https://github.com/trishume/syntect) - Syntax highlighting
* [fuzzy-matcher](https://github.com/lotabout/fuzzy-matcher) - Search algorithms
