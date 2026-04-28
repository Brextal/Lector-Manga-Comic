use eframe::egui;
use image::GenericImageView;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use unrar::Archive;

use crate::viewer::Viewer;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];
const CACHE_WINDOW: i32 = 2;
const PRELOAD_OFFSETS: &[i32] = &[-1, 1];

pub struct CbrViewer {
    page_num: i32,
    total_pages: i32,
    page_paths: Vec<String>,
    zoom: f32,
    file_path: String,
    file_name: String,
    texture_cache: HashMap<i32, (egui::TextureHandle, (usize, usize))>,
    error_message: Option<String>,
    page_input: String,
    temp_dir: PathBuf,
}

impl CbrViewer {
    pub fn new(file_path: &str) -> Option<Self> {
        let file_name = Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Create unique temp directory based on full path hash to avoid collisions
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        file_path.hash(&mut hasher);
        let path_hash = hasher.finish();

        let temp_dir = std::env::temp_dir()
            .join("lector-manga-comic")
            .join(format!("{}", path_hash));

        let _ = fs::create_dir_all(&temp_dir);

        // Clean any existing files in temp dir
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        let _ = fs::create_dir_all(&temp_dir);

        // Extrae todas las páginas al inicio
        let page_paths = Self::extract_all_pages(file_path, temp_dir.to_str().unwrap())?;
        if page_paths.is_empty() {
            return None;
        }

        Some(Self {
            page_num: 0,
            total_pages: page_paths.len() as i32,
            page_paths,
            zoom: 1.0,
            file_path: file_path.to_string(),
            file_name,
            texture_cache: HashMap::new(),
            error_message: None,
            page_input: String::new(),
            temp_dir,
        })
    }

    fn is_image_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        SUPPORTED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
    }

    fn extract_all_pages(file_path: &str, temp_dir: &str) -> Option<Vec<String>> {
        let mut archive = Archive::new(file_path.to_string())
            .extract_to(temp_dir.to_string())
            .ok()?;
        archive.process().ok()?;

        let mut pages = Vec::new();
        for entry in fs::read_dir(temp_dir).ok()? {
            let path = entry.ok()?.path();
            if let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) {
                if Self::is_image_file(&name) {
                    pages.push(path.to_string_lossy().to_string());
                }
            }
        }
        pages.sort();
        Some(pages)
    }

    fn read_page(&self, page_path: &str) -> Option<Vec<u8>> {
        fs::read(page_path).ok()
    }

    fn ensure_texture_loaded(&mut self, ctx: &egui::Context, page_idx: i32) {
        self.evict_old_cache_entries(page_idx);

        if self.texture_cache.contains_key(&page_idx) {
            self.error_message = None;
            return;
        }

        let page_path = match self.page_paths.get(page_idx as usize) {
            Some(p) => p.clone(),
            None => {
                self.error_message = Some(format!("Página {} no encontrada", page_idx));
                return;
            }
        };

        let data = match self.read_page(&page_path) {
            Some(d) => d,
            None => {
                self.error_message = Some(format!("No se pudo leer: {}", page_path));
                return;
            }
        };

        let img = match image::load_from_memory(&data) {
            Ok(i) => i,
            Err(e) => {
                self.error_message = Some(format!("Error al cargar imagen: {}", e));
                return;
            }
        };

        let (w, h) = img.dimensions();
        let rgba = img.to_rgba8();
        let bytes = rgba.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &bytes);

        let key = format!("page_{}", page_idx);
        let texture = ctx.load_texture(&key, color_image, Default::default());

        self.texture_cache.insert(page_idx, (texture, (w as usize, h as usize)));
        self.error_message = None;
    }

    fn evict_old_cache_entries(&mut self, current_page: i32) {
        let min_page = (current_page - CACHE_WINDOW).max(0);
        let max_page = (current_page + CACHE_WINDOW).min(self.total_pages - 1);

        self.texture_cache
            .retain(|page, _| *page >= min_page && *page <= max_page);
    }

    fn preload_neighbor_pages(&mut self, ctx: &egui::Context, current_page: i32) {
        for &offset in PRELOAD_OFFSETS {
            let idx = current_page + offset;
            if idx >= 0 && idx < self.total_pages && !self.texture_cache.contains_key(&idx) {
                self.ensure_texture_loaded(ctx, idx);
            }
        }
    }
}

impl Drop for CbrViewer {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

impl Viewer for CbrViewer {
    fn next_page(&mut self) {
        if self.page_num < self.total_pages - 1 {
            self.page_num += 1;
        }
    }

    fn prev_page(&mut self) {
        if self.page_num > 0 {
            self.page_num -= 1;
        }
    }

    fn set_page(&mut self, page: i32) {
        if page >= 0 && page < self.total_pages {
            self.page_num = page;
        }
    }

    fn current_page(&self) -> i32 {
        self.page_num
    }

    fn total_pages(&self) -> i32 {
        self.total_pages
    }

    fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.25).min(4.0);
    }

    fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.25).max(0.25);
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 4.0);
    }

    fn get_zoom(&self) -> f32 {
        self.zoom
    }

    fn zoom_percent(&self) -> i32 {
        (self.zoom * 100.0) as i32
    }

    fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
        let (go_back, page_input) = crate::viewer::render_navigation_bar(ui, self);
        self.page_input = page_input;

        if let Some(ref err) = self.error_message {
            ui.label(err);
        }

        ui.separator();

        self.ensure_texture_loaded(ctx, self.page_num);
        self.preload_neighbor_pages(ctx, self.page_num);

        if let Some((texture, (w, h))) = self.texture_cache.get(&self.page_num) {
            let display_w = (*w as f32 * self.zoom) as usize;
            let display_h = (*h as f32 * self.zoom) as usize;
            let scroll_size = egui::vec2(display_w as f32, display_h as f32);

            egui::ScrollArea::new([true, true]).show(ui, |ui| {
                ui.set_min_size(scroll_size);
                ui.add(egui::Image::from_texture(texture).max_size(scroll_size));

                // Scroll with arrow keys (same as mouse wheel)
                let mut scroll_delta = egui::Vec2::ZERO;
                if ui.input(|i| i.key_down(egui::Key::ArrowDown)) {
                    scroll_delta.y -= 15.0;  // Scroll down = content moves up
                }
                if ui.input(|i| i.key_down(egui::Key::ArrowUp)) {
                    scroll_delta.y += 15.0;  // Scroll up = content moves down
                }
                if scroll_delta != egui::Vec2::ZERO {
                    ui.scroll_with_delta(scroll_delta);
                }
            });
        } else {
            ui.spinner();
            ui.label("Cargando...");
        }

        let go_back = crate::viewer::handle_keyboard_shortcuts(ui, self) || go_back;

        go_back
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }

    fn get_file_name(&self) -> &str {
        &self.file_name
    }

    fn take_page_input(&mut self) -> String {
        std::mem::take(&mut self.page_input)
    }

    fn set_error_message(&mut self, msg: Option<String>) {
        self.error_message = msg;
    }
}
