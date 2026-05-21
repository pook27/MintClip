use arboard::{Clipboard, ImageData};
use std::sync::{Arc, Mutex};
use enigo::{Enigo, Key, KeyboardControllable};
use directories::ProjectDirs;
use eframe::egui;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    path::PathBuf,
    thread,
    time::{Duration, Instant, SystemTime},
};

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const MAX_HISTORY: usize = 50;

// --- STEP 1: THE DATA MODEL ---

#[derive(Serialize, Deserialize, Clone)]
enum ClipContent {
    Text(String),
    Image(PathBuf),
}

#[derive(Serialize, Deserialize, Clone)]
struct ClipItem {
    content: ClipContent,
    timestamp: SystemTime,
    is_pinned: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct ClipHistory {
    items: Vec<ClipItem>,
}

impl ClipHistory {
    fn get_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "mintclip", "mintclip").map(|d| d.config_dir().to_path_buf())
    }

    fn get_file_path() -> Option<PathBuf> {
        let dir = Self::get_dir()?;
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        Some(dir.join("history.json"))
    }

    fn load() -> Self {
        if let Some(path) = Self::get_file_path() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(history) = serde_json::from_str(&data) {
                    return history;
                }
            }
        }
        ClipHistory { items: Vec::new() }
    }

    fn save(&self) {
        if let Some(path) = Self::get_file_path() {
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, data);
            }
        }
    }

    fn add_new(&mut self, content: ClipContent) {
        let mut was_pinned = false;
        if let ClipContent::Text(ref new_text) = content {
            if let Some(pos) = self.items.iter().position(|item| {
                if let ClipContent::Text(ref existing_text) = item.content {
                    existing_text.trim() == new_text.trim()
                } else {
                    false
                }
            }) {
                was_pinned = self.items[pos].is_pinned;
                self.items.remove(pos);
            }
        }

        self.items.insert(
            0,
            ClipItem {
                content,
                timestamp: SystemTime::now(),
                is_pinned: was_pinned,
            },
        );

        let mut pinned = Vec::new();
        let mut unpinned = Vec::new();
        
        for item in self.items.drain(..) {
            if item.is_pinned {
                pinned.push(item);
            } else {
                unpinned.push(item);
            }
        }

        if unpinned.len() > MAX_HISTORY {
            for item in unpinned.drain(MAX_HISTORY..) {
                if let ClipContent::Image(path) = item.content {
                    let _ = fs::remove_file(path);
                }
            }
        }
        self.items = pinned;
        self.items.extend(unpinned);
    }
}

// --- HELPER FUNCTIONS ---

fn hash_image_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn save_image_to_disk(img_data: &ImageData) -> Option<PathBuf> {
    let config_dir = ClipHistory::get_dir()?;
    let img_dir = config_dir.join("images");
    if !img_dir.exists() {
        let _ = fs::create_dir_all(&img_dir);
    }

    let filename = format!(
        "{}.png",
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis()
    );
    let filepath = img_dir.join(filename);

    if let Some(img) = image::RgbaImage::from_raw(
        img_data.width as u32,
        img_data.height as u32,
        img_data.bytes.clone().into_owned(),
    ) {
        let _ = image::save_buffer(
            &filepath,
            &img,
            img_data.width as u32,
            img_data.height as u32,
            image::ColorType::Rgba8,
        );
        return Some(filepath);
    }
    None
}

fn cleanup_orphaned_images(history: &ClipHistory) {
    if let Some(dir) = ClipHistory::get_dir() {
        let img_dir = dir.join("images");
        if !img_dir.exists() { return; }

        // 1. Collect all valid image paths that are actually in our history
        let valid_paths: std::collections::HashSet<_> = history.items.iter()
            .filter_map(|item| {
                if let ClipContent::Image(path) = &item.content {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();

        // 2. Scan the hard drive and delete anything not in that valid list
        if let Ok(entries) = fs::read_dir(&img_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
                    if !valid_paths.contains(&path) {
                        let _ = fs::remove_file(&path);
                        println!("MintClip GC: Deleted orphaned image {:?}", path);
                    }
                }
            }
        }
    }
}

fn detect_content_type(text: &str) -> (&'static str, egui::Color32) {
    let t = text.trim();
    if t.starts_with("http://") || t.starts_with("https://") {
        ("URL", egui::Color32::from_rgb(100, 180, 255))
    } else if t.contains('@') && !t.contains(' ') && t.contains('.') {
        ("Email", egui::Color32::from_rgb(255, 200, 100))
    } else if !t.contains(' ') && (t.starts_with("ghp_") || t.starts_with("gho_") || (t.len() > 24 && t.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'))) {
        ("Token", egui::Color32::from_rgb(255, 100, 100))
    } else if (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']')) {
        ("JSON", egui::Color32::from_rgb(150, 255, 150))
    } else if (t.starts_with('/') && !t.starts_with("//")) || t.starts_with("~/") {
        ("Path", egui::Color32::from_rgb(255, 150, 255))
    } else if t.contains("fn ") || t.contains("let ") || t.contains("sudo ") || t.contains("const ") 
        || t.contains("import ") || t.contains("def ") || t.contains("class ") 
        || t.contains("var ") || t.contains("print(") || t.contains("console.log(")
        || t.contains("println!(") || t.contains("public ") || t.contains("<?php")
        || (t.contains("if ") && t.contains('{')) || (t.contains("for ") && t.contains('{')) {
        ("Code", egui::Color32::from_rgb(100, 255, 150))
    } else {
        ("Text", egui::Color32::GRAY)
    }
}

fn fix_bidi_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.split('\n') {
        let info = unicode_bidi::BidiInfo::new(line, Some(unicode_bidi::Level::ltr()));
        if let Some(para) = info.paragraphs.first() {
            let visual = info.reorder_line(para, para.range.clone());
            result.push_str(&visual);
        }
        result.push('\n');
    }
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

// --- STEP 2: THE BACKGROUND DAEMON ---

fn run_daemon() {
    println!("MintClip daemon started. Listening for clipboard changes...");
    
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("CRITICAL ERROR: Failed to connect to system clipboard. Retrying later... ({})", e);
            std::thread::sleep(Duration::from_secs(3));
            return;
        }
    };

    // Use unwrap_or_default() so if the clipboard is empty on boot, it just starts with an empty string
    let mut last_copied_text = clipboard.get_text().unwrap_or_default();
    let mut last_image_hash: u64 = 0;
    
    loop {
        if let Ok(current_text) = clipboard.get_text() {
            let current_text = current_text.trim().to_string();
            if !current_text.is_empty() && current_text != last_copied_text {
                let mut history = ClipHistory::load();

                history.add_new(ClipContent::Text(current_text.clone()));
                history.save();
                last_copied_text = current_text;
            }
        }

        if let Ok(current_image) = clipboard.get_image() {
            let current_hash = hash_image_bytes(&current_image.bytes);
            if current_hash != last_image_hash {
                if let Some(path) = save_image_to_disk(&current_image) {
                    let mut history = ClipHistory::load(); 

                    history.add_new(ClipContent::Image(path));
                    history.save();
                }
                last_image_hash = current_hash;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

// --- STEP 3: THE VISUAL UI ---

struct MintClipUI {
    history: ClipHistory,
    search_query: String,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    copied_status: Option<(usize, Instant)>,
    should_paste: Arc<Mutex<bool>>,
}

impl MintClipUI {
    fn new(cc: &eframe::CreationContext<'_>, should_paste: Arc<Mutex<bool>>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = egui::Rounding::same(14.0);
        visuals.panel_fill = egui::Color32::from_rgb(16, 18, 24);
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 40, 52));
        visuals.selection.bg_fill = egui::Color32::from_rgb(55, 95, 160);
        cc.egui_ctx.set_visuals(visuals);

        let mut fonts = egui::FontDefinitions::default();
        let font_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"
        ];

        for path in font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "system_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    vec.insert(0, "system_font".to_owned());
                }
                if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    vec.insert(0, "system_font".to_owned());
                }
                break; 
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        let history = ClipHistory::load();
        cleanup_orphaned_images(&history);

        Self {
            history,
            search_query: String::new(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            copied_status: None,
            should_paste,
        }
    }

    fn highlight_text(syntax_set: &SyntaxSet, theme_set: &ThemeSet, text: &str, category: &str) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        
        if category != "Code" && category != "JSON" && category != "Token" {
            let bidi_fixed_text = fix_bidi_text(text);
            job.append(
                &bidi_fixed_text,
                0.0,
                egui::TextFormat {
                    color: egui::Color32::LIGHT_GRAY,
                    font_id: egui::FontId::proportional(14.0),
                    ..Default::default()
                },
            );
            return job;
        }

        let syntax = if category == "JSON" {
            syntax_set.find_syntax_by_extension("json")
                .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
        } else {
            syntax_set.find_syntax_by_first_line(text)
                .or_else(|| syntax_set.find_syntax_by_extension("rs"))
                .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
        };

        let theme = &theme_set.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);

        for line in LinesWithEndings::from(text) {
            let ranges = h.highlight_line(line, syntax_set).unwrap_or_default();
            for &(style, text_segment) in ranges.iter() {
                let color = egui::Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                job.append(
                    text_segment,
                    0.0,
                    egui::TextFormat {
                        color,
                        font_id: egui::FontId::monospace(14.0),
                        ..Default::default()
                    },
                );
            }
        }
        job
    }
}

impl eframe::App for MintClipUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        if let Some((_, time_clicked)) = self.copied_status {
            if time_clicked.elapsed() > Duration::from_millis(150) {
                // Flag that we want to trigger a paste
                if let Ok(mut sp) = self.should_paste.lock() {
                    *sp = true;
                }
                // Tell eframe to safely close the window
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                // Ensure egui repaints continually so we actually see the feedback text
                ctx.request_repaint(); 
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            
            // ── Header ──────────────────────────────────────────────────────
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("MintClip")
                            .size(15.0)
                            .color(egui::Color32::from_rgb(110, 190, 140))
                            .strong(),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new("🗑️")
                                .size(20.0)
                                .color(egui::Color32::from_rgb(90, 95, 115)),
                        )
                        .frame(false),
                    ).clicked() {
                        self.history.items.retain(|item| item.is_pinned);
                        self.history.save();
                    }
                });
            });
            ui.add_space(8.0);

            // ── Search bar ───────────────────────────────────────────────────
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(24, 27, 36))
                .rounding(8.0)
                .inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 7.0, bottom: 7.0 })
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(42, 47, 62)))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🔍")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(85, 92, 115)),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("search…")
                                .frame(false)
                                .desired_width(f32::INFINITY),
                        );
                    });
                });

            ui.add_space(10.0);

            let mut needs_save = false;
            let mut item_to_delete = None;

            let matcher = SkimMatcherV2::default();
            // (item_index, fuzzy_score, is_category_match)
            let mut display_items: Vec<(usize, i64, bool)> = Vec::new();

            let query_lower = self.search_query.to_lowercase();

            for (i, item) in self.history.items.iter().enumerate() {
                if self.search_query.is_empty() {
                    display_items.push((i, 0, false));
                } else {
                    match &item.content {
                        ClipContent::Text(text) => {
                            let (tag, _) = detect_content_type(text);
                            
                            // 1. Check for Category Match (Fuzzy match against the tag name)
                            let is_category_match = matcher
                                .fuzzy_match(&tag.to_lowercase(), &query_lower)
                                .is_some();
                                
                            // 2. Check for Text Match
                            let text_score = matcher.fuzzy_match(text, &self.search_query);

                            if is_category_match || text_score.is_some() {
                                // If it matches the category name, we flag it to push it to the top
                                display_items.push((i, text_score.unwrap_or(0), is_category_match));
                            }
                        }
                        ClipContent::Image(_) => {
                            if matcher.fuzzy_match("image", &query_lower).is_some() {
                                display_items.push((i, 0, true));
                            }
                        }
                    }
                }
            }

            if !self.search_query.is_empty() {
                display_items.sort_by(|a, b| {
                    let item_a = &self.history.items[a.0];
                    let item_b = &self.history.items[b.0];

                    // Priority 1: Pinned status
                    item_b.is_pinned.cmp(&item_a.is_pinned)
                        // Priority 2: Is it a Category Match? (e.g., searching "Token")
                        .then_with(|| b.2.cmp(&a.2))
                        // Priority 3: Fuzzy text match score
                        .then_with(|| b.1.cmp(&a.1))
                });
            }

            let syntax_set = &self.syntax_set;
            let theme_set = &self.theme_set;

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for &(original_idx, _, _) in &display_items {
                    let item = &mut self.history.items[original_idx];
                    let accent_color = match &item.content {
                        ClipContent::Text(text) => detect_content_type(text).1,
                        ClipContent::Image(_) => egui::Color32::from_rgb(255, 140, 90),
                    };
                    
                    // See if this specific item is the one that was just copied
                    let is_recently_copied = self.copied_status.map_or(false, |(idx, _)| idx == original_idx);

                    let card_id = ui.id().with(original_idx);
                    let is_hovered = ctx.data(|d| d.get_temp::<bool>(card_id).unwrap_or(false));

                    // 2. Set dynamic colors
                    let bg = if is_hovered {
                        egui::Color32::from_rgb(30, 34, 46)
                    } else {
                        egui::Color32::from_rgb(22, 25, 33)
                    };

                    let stroke = if is_hovered {
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(58, 76, 112))
                    } else {
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(32, 36, 48))
                    };

                    // 3. Draw the frame and capture it in a variable
                    let frame_response = egui::Frame::none()
                        .fill(bg)
                        .stroke(stroke)
                        .rounding(8.0)
                        .inner_margin(egui::Margin { left: 16.0, right: 10.0, top: 10.0, bottom: 10.0 })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    
                                    // 1. Delete Button — clean ✕ symbol
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("✕")
                                                .size(11.0)
                                                .color(egui::Color32::from_rgb(100, 105, 125)),
                                        )
                                        .frame(false),
                                    ).on_hover_text("Delete").clicked() {
                                        item_to_delete = Some(original_idx);
                                    }

                                    // 2. Pin Button — geometric diamond
                                    let (pin_icon, pin_color) = if item.is_pinned {
                                        ("◆", egui::Color32::from_rgb(85, 165, 250))
                                    } else {
                                        ("◇", egui::Color32::from_rgb(75, 80, 100))
                                    };
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(pin_icon)
                                                .size(11.0)
                                                .color(pin_color),
                                        )
                                        .frame(false),
                                    ).on_hover_text(if item.is_pinned { "Unpin" } else { "Pin" }).clicked() {
                                        item.is_pinned = !item.is_pinned;
                                        needs_save = true;
                                    }

                                    // 3. Relative timestamp
                                    let time_str = {
                                        let elapsed = item.timestamp.elapsed().unwrap_or_default();
                                        let s = elapsed.as_secs();
                                        if s < 60 { "just now".to_string() }
                                        else if s < 3600 { format!("{}m ago", s / 60) }
                                        else if s < 86400 { format!("{}h ago", s / 3600) }
                                        else {
                                            let d = s / 86400;
                                            if d == 1 { "yesterday".to_string() } else { format!("{}d ago", d) }
                                        }
                                    };
                                    ui.label(egui::RichText::new(time_str).size(10.0).color(egui::Color32::from_rgb(72, 78, 98)));
                                    
                                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        
                                        if is_recently_copied {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(8.0);
                                                ui.label(
                                                    egui::RichText::new("✓ copied")
                                                        .color(egui::Color32::from_rgb(100, 200, 140))
                                                        .size(13.0),
                                                );
                                                ui.add_space(8.0);
                                            });
                                        }
                                        else {
                                            // Render the standard text or image
                                            match &item.content {
                                                ClipContent::Text(raw_text) => {
                                                    let (tag, color) = detect_content_type(raw_text);
                                                    ui.vertical(|ui| {
                                                        // Pill badge for content type
                                                        egui::Frame::none()
                                                            .fill(egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 25))
                                                            .rounding(4.0)
                                                            .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 2.0, bottom: 2.0 })
                                                            .show(ui, |ui| {
                                                                ui.label(egui::RichText::new(tag).size(9.0).color(color));
                                                            });
                                                        
                                                        let display_text = if raw_text.chars().count() > 300 {
                                                            format!("{}...", raw_text.chars().take(300).collect::<String>())
                                                        } else {
                                                            raw_text.to_string()
                                                        };

                                                        let layout_job = Self::highlight_text(syntax_set, theme_set, &display_text, tag);

                                                        egui::ScrollArea::vertical()
                                                            .id_source(original_idx) // Required so egui doesn't get confused in a loop
                                                            .max_height(150.0)
                                                            .show(ui, |ui| {
                                                                let copy_btn = egui::Button::new(layout_job).wrap(true).frame(false);
                                                                
                                                                if ui.add_sized([ui.available_width(), 0.0], copy_btn).on_hover_text("Click to copy").clicked() {
                                                                    if let Ok(mut ctx_clip) = Clipboard::new() {
                                                                        let _ = ctx_clip.set_text(raw_text.clone());
                                                                    }
                                                                    // Trigger the visual delay feedback!
                                                                    self.copied_status = Some((original_idx, Instant::now()));
                                                                }
                                                            });
                                                        
                                                    });
                                                }
                                                ClipContent::Image(path) => {
                                                    ui.vertical(|ui| {
                                                        let img_color = egui::Color32::from_rgb(255, 140, 90);
                                                        egui::Frame::none()
                                                            .fill(egui::Color32::from_rgba_unmultiplied(255, 140, 90, 25))
                                                            .rounding(4.0)
                                                            .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 2.0, bottom: 2.0 })
                                                            .show(ui, |ui| {
                                                                ui.label(egui::RichText::new("image").size(9.0).color(img_color));
                                                            });

                                                        let img_uri = format!("file://{}", path.display());
                                                        let response = ui.add(
                                                            egui::Image::new(&img_uri)
                                                            // Kept exactly as you requested!
                                                            .fit_to_original_size(1.0)
                                                            .max_width(280.0)
                                                            .max_height(200.0)
                                                            .rounding(5.0)
                                                            .sense(egui::Sense::click())
                                                        );

                                                        if response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text("Click to copy image").clicked() {
                                                            if let Ok(mut ctx_clip) = Clipboard::new() {
                                                                if let Ok(img) = image::open(path) {
                                                                    let rgba = img.into_rgba8();
                                                                    let (w, h) = rgba.dimensions();
                                                                    let clip_img = ImageData {
                                                                        width: w as usize,
                                                                        height: h as usize,
                                                                        bytes: std::borrow::Cow::Owned(rgba.into_raw()),
                                                                    };
                                                                    let _ = ctx_clip.set_image(clip_img);
                                                                }
                                                            }
                                                            // Trigger the visual delay feedback!
                                                            self.copied_status = Some((original_idx, Instant::now()));
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                        }).response;
                    // Draw left accent bar keyed to content type
                    {
                        let rect = frame_response.rect;
                        let accent_rect = egui::Rect::from_min_max(
                            rect.min + egui::vec2(1.0, 7.0),
                            egui::pos2(rect.min.x + 4.0, rect.max.y - 7.0),
                        );
                        let bar_color = egui::Color32::from_rgba_unmultiplied(
                            accent_color.r(), accent_color.g(), accent_color.b(), 180,
                        );
                        ui.painter().rect_filled(accent_rect, egui::Rounding::same(2.0), bar_color);
                    }

                    let interact_response = ui.interact(frame_response.rect, card_id, egui::Sense::hover());
                    ctx.data_mut(|d| d.insert_temp(card_id, interact_response.hovered()));
                    ui.add_space(5.0);
                }

                if let Some(idx) = item_to_delete {
                    let removed_item = self.history.items.remove(idx);
                    
                    // Wipe the file from the hard drive if it was an image
                    if let ClipContent::Image(path) = removed_item.content {
                        let _ = fs::remove_file(path);
                    }
                    needs_save = true;
                }

                if needs_save {
                    self.history.save();
                }
            });
        });
    }
}

// --- STEP 4: SYSTEM INTEGRATION ---

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("--daemon")) {
        let daemon_pid_file = std::env::temp_dir().join("mintclip_daemon.pid");
        if let Ok(pid_str) = fs::read_to_string(&daemon_pid_file) {
            let pid = pid_str.trim();
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            if let Ok(cmdline) = fs::read_to_string(cmdline_path) {
                if cmdline.contains("mintclip") && cmdline.contains("--daemon") {
                    println!("Old daemon found (PID: {}). Restarting with new code...", pid);
                    let _ = std::process::Command::new("kill").arg("-9").arg(pid).status();
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
        let _ = fs::write(&daemon_pid_file, std::process::id().to_string());
        run_daemon();
    } else {
        let pid_file = std::env::temp_dir().join("mintclip_ui.pid");
        if let Ok(pid_str) = fs::read_to_string(&pid_file) {
            let pid = pid_str.trim();
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            if let Ok(cmdline) = fs::read_to_string(cmdline_path) {
                if cmdline.contains("mintclip") && !cmdline.contains("--daemon") {
                    if let Ok(status) = std::process::Command::new("kill").arg(pid).status() {
                        if status.success() {
                            let _ = fs::remove_file(&pid_file);
                            return; 
                        }
                    }
                }
            }
        }
        
        let _ = fs::write(&pid_file, std::process::id().to_string());

        let icon_data = if let Ok(image_bytes) = fs::read("assets/icon.png") {
            if let Ok(image) = image::load_from_memory(&image_bytes) {
                let rgba = image.into_rgba8();
                let (width, height) = rgba.dimensions();
                Some(egui::IconData {
                    rgba: rgba.into_raw(),
                    width,
                    height,
                })
            } else { None }
        } else { None };

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_decorations(true)          
                .with_always_on_top()             
                .with_inner_size([450.0, 600.0])
                .with_icon(icon_data.unwrap_or_default()),
            ..Default::default()
        };

        let should_paste = Arc::new(Mutex::new(false));
        let app_should_paste = Arc::clone(&should_paste);

        // Pass the state into the App
        if let Err(e) = eframe::run_native(
            "MintClip",
            options,
            Box::new(move |cc| Box::new(MintClipUI::new(cc, app_should_paste))), 
        ) {
            eprintln!("Failed to launch MintClip UI: {}", e);
        }
        
        // NEW: The window is now closed. Let's paste!
        if *should_paste.lock().unwrap() {
            // Wait 100ms for the OS to restore focus to your previous window
            std::thread::sleep(Duration::from_millis(100));

            let mut enigo = Enigo::new();
            enigo.key_down(Key::Control);
            enigo.key_down(Key::Shift);
            enigo.key_click(Key::Layout('v'));
            enigo.key_up(Key::Shift);
            enigo.key_up(Key::Control);
        }
    }
}
