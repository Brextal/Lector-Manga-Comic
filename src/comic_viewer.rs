use eframe::egui;
use image::GenericImageView;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::viewer::Viewer;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];
const CACHE_WINDOW: i32 = 2;
const PRELOAD_OFFSETS: &[i32] = &[-1, 1];

pub struct ComicViewer {
    page_num: i32,
    total_pages: i32,
    page_names: Vec<String>,
    zoom: f32,
    file_path: String,
    file_name: String,
    texture_cache: HashMap<(i32, i32), (egui::TextureHandle, (usize, usize))>,
    archive: Option<ZipArchive<File>>,
    error_message: Option<String>,
}

impl ComicViewer {
    pub fn new(file_path: &str) -> Option<Self> {
        let file_path_str = Path::new(file_path);
        let file_name = file_path_str
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let actual_path = Self::urlencoding_decode(file_path);

        let page_names = Self::list_zip_contents(&actual_path)?;

        if page_names.is_empty() {
            return None;
        }

        let total_pages = page_names.len() as i32;

        let file = File::open(&actual_path).ok()?;
        let archive = ZipArchive::new(file).ok()?;

        Some(Self {
            page_num: 0,
            total_pages,
            page_names,
            zoom: 1.0,
            file_path: actual_path,
            file_name,
            texture_cache: HashMap::new(),
            archive: Some(archive),
            error_message: None,
        })
    }

    fn urlencoding_decode(s: &str) -> String {
        if !s.contains('%') {
            return s.to_string();
        }
        let mut result = s.to_string();
        result = result.replace("%23", "#");
        result = result.replace("%28", "(");
        result = result.replace("%29", ")");
        result = result.replace("%20", " ");
        result
    }

    fn is_image_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        SUPPORTED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
    }

    fn extract_page_number(name: &str) -> Option<u32> {
        let name_lower = name.to_lowercase();
        let nums: Vec<u32> = name_lower
            .chars()
            .collect::<Vec<_>>()
            .split(|c| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.iter().collect::<String>().parse().ok())
            .collect();

        nums.last().copied()
    }

    fn natural_sort_key(name: &str) -> (Option<u32>, String) {
        (Self::extract_page_number(name), name.to_lowercase())
    }

    fn list_zip_contents(file_path: &str) -> Option<Vec<String>> {
        let file = File::open(file_path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;

        let mut page_names: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let name = archive.by_index(i).ok()?.name().to_string();
                if Self::is_image_file(&name) {
                    Some(name)
                } else {
                    None
                }
            })
            .filter(|name| !name.starts_with("__MACOSX"))
            .collect();

        page_names.sort_by(|a, b| Self::natural_sort_key(a).cmp(&Self::natural_sort_key(b)));

        Some(page_names)
    }

    fn read_page_from_zip(&mut self, page_name: &str) -> Option<Vec<u8>> {
        let archive = self.archive.as_mut()?;
        let mut zip_file = archive.by_name(page_name).ok()?;
        let mut data = Vec::new();
        zip_file.read_to_end(&mut data).ok()?;
        Some(data)
    }

    fn ensure_texture_loaded(&mut self, ctx: &egui::Context, page_idx: i32, zoom_percent: i32) {
        self.evict_old_cache_entries(page_idx);

        let cache_key = (page_idx, zoom_percent);
        if self.texture_cache.contains_key(&cache_key) {
            self.error_message = None;
            return;
        }

        let page_name = match self.page_names.get(page_idx as usize) {
            Some(n) => n.clone(),
            None => {
                self.error_message = Some(format!("Página {} no encontrada", page_idx));
                return;
            }
        };

        let data = match self.read_page_from_zip(&page_name) {
            Some(d) => d,
            None => {
                self.error_message = Some(format!("No se pudo leer: {}", page_name));
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
        let w_usize = w as usize;
        let h_usize = h as usize;
        let rgba = img.to_rgba8();
        let bytes = rgba.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([w_usize, h_usize], &bytes);

        let key = format!("page_{}_{}", page_idx, zoom_percent);
        let texture = ctx.load_texture(&key, color_image, Default::default());

        self.texture_cache
            .insert(cache_key, (texture, (w_usize, h_usize)));
        self.error_message = None;
    }

    fn get_cached_dims(&self, page_idx: i32, zoom_percent: i32) -> Option<(usize, usize)> {
        self.texture_cache
            .get(&(page_idx, zoom_percent))
            .map(|(_, dims)| *dims)
    }

    fn evict_old_cache_entries(&mut self, current_page: i32) {
        let min_page = (current_page - CACHE_WINDOW).max(0);
        let max_page = (current_page + CACHE_WINDOW).min(self.total_pages - 1);

        self.texture_cache
            .retain(|(page, _), _| *page >= min_page && *page <= max_page);
    }

    fn preload_neighbor_pages(
        &mut self,
        ctx: &egui::Context,
        current_page: i32,
        zoom_percent: i32,
    ) {
        let total = self.total_pages;

        for &offset in PRELOAD_OFFSETS {
            let idx = current_page + offset;
            if idx >= 0 && idx < total && !self.texture_cache.contains_key(&(idx, zoom_percent)) {
                self.ensure_texture_loaded(ctx, idx, zoom_percent);
            }
        }
    }
}

impl Viewer for ComicViewer {
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
        let mut go_back = false;

        ui.horizontal(|ui| {
            if ui.button("Abrir archivo").clicked() {
                go_back = true;
            }
            ui.add_space(10.0);

            if ui.button("Ant.").clicked() {
                self.prev_page();
            }
            ui.label(format!(
                "Pagina {} / {}",
                self.current_page() + 1,
                self.total_pages()
            ));
            if ui.button("Sig.").clicked() {
                self.next_page();
            }
            ui.add_space(20.0);
            if ui.button("-").clicked() {
                self.zoom_out();
            }
            ui.label(format!("{}%", self.zoom_percent()));
            if ui.button("+").clicked() {
                self.zoom_in();
            }
        });

        ui.separator();

        let zoom_percent = (self.zoom * 100.0) as i32;
        self.ensure_texture_loaded(ctx, self.page_num, zoom_percent);
        self.preload_neighbor_pages(ctx, self.page_num, zoom_percent);

        let (w, h) = self
            .get_cached_dims(self.page_num, zoom_percent)
            .unwrap_or((800, 600));

        let zoom = self.zoom;
        let display_w = (w as f32 * zoom) as usize;
        let display_h = (h as f32 * zoom) as usize;
        let scroll_size = egui::vec2(display_w as f32, display_h as f32);

        if let Some((texture, _)) = self.texture_cache.get(&(self.page_num, zoom_percent)) {
            egui::ScrollArea::new([true, true]).show(ui, |ui: &mut egui::Ui| {
                ui.set_min_size(scroll_size);
                let img = egui::Image::from_texture(texture).max_size(scroll_size);
                ui.add(img);
            });
        } else if let Some(ref err) = self.error_message {
            ui.label(err);
        } else {
            ui.spinner();
            ui.label("Cargando...");
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.prev_page();
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.next_page();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Plus))
            || ui.input(|i| i.key_pressed(egui::Key::Equals))
        {
            self.zoom_in();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Minus)) {
            self.zoom_out();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Q)) {
            go_back = true;
        }

        go_back
    }

    fn get_file_path(&self) -> &str {
        &self.file_path
    }

    fn get_file_name(&self) -> &str {
        &self.file_name
    }
}
