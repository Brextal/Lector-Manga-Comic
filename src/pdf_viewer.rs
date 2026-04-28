use cairo::{Context, Format, ImageSurface};
use eframe::egui;
use poppler::Document;

use crate::viewer::Viewer;

/// Encode special characters in file path for URI
/// Especially handles: #, spaces, etc.
fn encode_file_path(path: &str) -> String {
    let mut encoded = String::new();
    for c in path.chars() {
        match c {
            '#' => encoded.push_str("%23"),
            '%' => encoded.push_str("%25"),
            ' ' => encoded.push_str("%20"),
            '(' => encoded.push_str("%28"),
            ')' => encoded.push_str("%29"),
            _ => encoded.push(c),
        }
    }
    encoded
}

pub struct PdfViewer {
    doc: Document,
    page_num: i32,
    zoom: f32,
    surface: Option<ImageSurface>,
    texture: Option<egui::TextureHandle>,
    last_render_size: Option<(i32, i32)>,
    file_path: String,
    file_name: String,
    pending_page: i32,
    pending_zoom: f32,
    is_dirty: bool,
    error_message: Option<String>,
    page_input: String,
}

impl PdfViewer {
    pub fn new(file_path: &str) -> Option<Self> {
        // Strip file:// prefix if present
        let path_str = if file_path.starts_with("file://") {
            file_path.strip_prefix("file://").unwrap_or(file_path)
        } else {
            file_path
        };

        // Encode special characters for URI (especially #)
        let file_uri = format!("file://{}", encode_file_path(path_str));
        let doc = Document::from_file(&file_uri, None).ok()?;

        if doc.n_pages() == 0 {
            return None;
        }

        let file_path_str = path_str.to_string();

        let file_name = std::path::Path::new(&file_path_str)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut viewer = Self {
            doc,
            page_num: 0,
            zoom: 1.0,
            surface: None,
            texture: None,
            last_render_size: None,
            file_path: file_path_str,
            file_name,
            pending_page: 0,
            pending_zoom: 1.0,
            is_dirty: true,
    error_message: None,
    page_input: String::new(),
};

        viewer.render_page_sync();
        Some(viewer)
    }

    fn render_page_sync(&mut self) {
        if !self.is_dirty {
            self.error_message = None;
            return;
        }

        let page = match self.doc.page(self.pending_page) {
            Some(p) => p,
            None => {
                self.error_message = Some("Página no encontrada".to_string());
                return;
            }
        };

        let (page_w, page_h) = page.size();
        let zoom_capped = self.pending_zoom.min(4.0);
        let render_w = (page_w as f32 * zoom_capped) as i32;
        let render_h = (page_h as f32 * zoom_capped) as i32;

        let surface = match ImageSurface::create(Format::ARgb32, render_w, render_h) {
            Ok(s) => s,
            Err(e) => {
                self.error_message = Some(format!("Error al crear superficie: {}", e));
                return;
            }
        };

        let cr = match Context::new(&surface) {
            Ok(c) => c,
            Err(e) => {
                self.error_message = Some(format!("Error al crear contexto: {}", e));
                return;
            }
        };

        cr.scale(zoom_capped as f64, zoom_capped as f64);
        page.render(&cr);

        self.surface = Some(surface);
        self.texture = None;
        self.last_render_size = Some((render_w, render_h));
        self.is_dirty = false;
        self.error_message = None;
    }

    pub fn next_page(&mut self) {
        if self.page_num < self.doc.n_pages() - 1 {
            self.page_num += 1;
            self.pending_page = self.page_num;
            self.pending_zoom = self.zoom;
            self.is_dirty = true;
        }
    }

    pub fn prev_page(&mut self) {
        if self.page_num > 0 {
            self.page_num -= 1;
            self.pending_page = self.page_num;
            self.pending_zoom = self.zoom;
            self.is_dirty = true;
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.25).min(4.0);
        self.pending_zoom = self.zoom;
        self.is_dirty = true;
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.25).max(0.25);
        self.pending_zoom = self.zoom;
        self.is_dirty = true;
    }

    pub fn current_page(&self) -> i32 {
        self.page_num
    }

    pub fn total_pages(&self) -> i32 {
        self.doc.n_pages()
    }

    pub fn zoom_percent(&self) -> i32 {
        (self.zoom * 100.0) as i32
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_page(&mut self, page: i32) {
        if page >= 0 && page < self.doc.n_pages() {
            self.page_num = page;
            self.pending_page = page;
            self.is_dirty = true;
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 4.0);
        self.pending_zoom = self.zoom;
        self.is_dirty = true;
    }

    fn get_render_info(&self) -> Option<(usize, usize, bool)> {
        let surface = self.surface.as_ref()?;
        let width = surface.width() as usize;
        let height = surface.height() as usize;

        let should_rebuild = if self.texture.is_none() {
            true
        } else if let Some((last_w, last_h)) = self.last_render_size {
            last_w != width as i32 || last_h != height as i32
        } else {
            true
        };

        Some((width, height, should_rebuild))
    }

    fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
        let (go_back, page_input) = crate::viewer::render_navigation_bar(ui, self);
        self.page_input = page_input;

        if let Some(ref err) = self.error_message {
            ui.label(err);
        }

        ui.separator();

        self.render_page_sync();

        let render_info = self.get_render_info();

        let (dims, should_rebuild) = match render_info {
            Some((w, h, r)) => ((w, h), r),
            None => ((800, 1200), false),
        };

        // Rebuild texture if needed or if zoom changed (is_dirty is true)
        if should_rebuild || self.is_dirty {
            if let Some(surface) = &mut self.surface {
                if let Ok(data) = surface.data() {
                    let mut bytes = data.to_vec();
                    for chunk in bytes.chunks_mut(4) {
                        chunk.swap(0, 2);
                    }
                    let image = egui::ColorImage::from_rgba_unmultiplied([dims.0, dims.1], &bytes);
                    self.texture = Some(ctx.load_texture("page", image, Default::default()));
                }
            }
        }

        // Apply zoom to display size
        let display_w = (dims.0 as f32 * self.zoom) as usize;
        let display_h = (dims.1 as f32 * self.zoom) as usize;
        let scroll_size = egui::vec2(display_w as f32, display_h as f32);

        if let Some(ref texture) = self.texture {
            egui::ScrollArea::new([true, true]).show(ui, |ui: &mut egui::Ui| {
                ui.set_min_size(scroll_size);
                ui.add(egui::Image::new(texture).max_size(scroll_size));

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
        } else if self.is_dirty {
            ui.spinner();
            ui.label("Cargando...");
        }

        let go_back = crate::viewer::handle_keyboard_shortcuts(ui, self) || go_back;

        go_back
    }
}

impl Viewer for PdfViewer {
    fn next_page(&mut self) {
        self.next_page();
    }
    fn prev_page(&mut self) {
        self.prev_page();
    }
    fn set_page(&mut self, page: i32) {
        self.set_page(page);
    }
    fn current_page(&self) -> i32 {
        self.page_num
    }
    fn total_pages(&self) -> i32 {
        self.doc.n_pages()
    }
    fn zoom_in(&mut self) {
        self.zoom_in();
    }
    fn zoom_out(&mut self) {
        self.zoom_out();
    }
    fn set_zoom(&mut self, zoom: f32) {
        self.set_zoom(zoom);
    }
    fn get_zoom(&self) -> f32 {
        self.zoom
    }
    fn zoom_percent(&self) -> i32 {
        (self.zoom * 100.0) as i32
    }
    fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
        self.render(ctx, ui)
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
