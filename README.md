Here is a punchy description, the step-by-step guide to changing the logo, and a complete, professional `README.md` for your GitHub repository.

### Project Description

**MintClip** is a blazing-fast, natively-rendered clipboard manager for Linux built entirely in Rust. Designed to bring the fluid `Win+V` experience to Linux, it features a background daemon that silently logs your clipboard history, and a lightweight GUI that offers fuzzy searching, image support, and automatic syntax highlighting for developer workflows.

---

### How to Change the Application Logo

To give MintClip a custom icon, you actually need to change it in **two** places: the internal app window (what `egui` renders) and the external OS shortcut (what Linux Mint renders in the taskbar and application menu).

#### 1. Changing the Window Icon (`egui`)

First, get a `.png` file of your logo and put it in your project folder (e.g., `assets/icon.png`). Then, update your `fn main()` function to load this image into `eframe`:

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("--daemon")) {
        // ... (Keep your daemon logic here)
    } else {
        // ... (Keep your PID toggle logic here)
        
        // NEW: Load your custom icon!
        let icon_data = if let Ok(image_bytes) = fs::read("assets/icon.png") {
            if let Ok(image) = image::load_from_memory(&image_bytes) {
                let rgba = image.into_rgba8();
                let (width, height) = rgba.dimensions();
                Some(eframe::icon_data::from_rgba_unmultiplied(
                    rgba.into_raw(),
                    width,
                    height,
                ).unwrap())
            } else { None }
        } else { None };

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_decorations(true)          
                .with_always_on_top()             
                .with_inner_size([450.0, 600.0])
                .with_icon(icon_data.unwrap_or_default()), // Inject the icon here
            ..Default::default()
        };

        eframe::run_native(
            "MintClip",
            options,
            Box::new(|cc| Box::new(MintClipUI::new(cc))), 
        ).unwrap();
    }
}

```

#### 2. Changing the Linux System Icon (`.desktop` file)

Linux Mint uses `.desktop` files to define how apps look in your application launcher and taskbar.

1. Create a file named `mintclip.desktop` in `~/.local/share/applications/`
2. Open it in a text editor and paste this block, replacing the paths with your actual paths:

```ini
[Desktop Entry]
Name=MintClip
Comment=Native Linux Clipboard Manager
Exec=/home/or/Projects/mintclip/target/release/mintclip
Icon=/home/or/Projects/mintclip/assets/icon.png
Terminal=false
Type=Application
Categories=Utility;

```

Once you save this, MintClip will have your custom logo across your entire operating system!

---

### README.md

You can copy and paste this directly into a `README.md` file in your project root.

```markdown
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
