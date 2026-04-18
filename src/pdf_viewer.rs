use cairo::{Context, Format, ImageSurface};
use eframe::egui;
use poppler::Document;

pub struct PdfViewer {
    doc: Document,
    page_num: i32,
    surface: Option<ImageSurface>,
    zoom: f32,
    texture: Option<egui::TextureHandle>,
    last_render_size: Option<(i32, i32)>,
}

impl PdfViewer {
    pub fn new(file_path: &str) -> Option<Self> {
        let doc = Document::from_file(file_path, None).ok()?;

        if doc.n_pages() == 0 {
            return None;
        }

        let mut viewer = Self {
            doc,
            page_num: 0,
            surface: None,
            zoom: 1.0,
            texture: None,
            last_render_size: None,
        };

        viewer.render_page();
        Some(viewer)
    }

    pub fn render_page(&mut self) {
        if let Some(page) = self.doc.page(self.page_num) {
            let base_w = 800;
            let base_h = 1200;

            let zoom_capped = self.zoom.min(4.0);
            let render_w = (base_w as f32 * zoom_capped) as i32;
            let render_h = (base_h as f32 * zoom_capped) as i32;

            let surface = match ImageSurface::create(Format::ARgb32, render_w, render_h) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error al crear superficie: {}", e);
                    return;
                }
            };

            let cr = match Context::new(&surface) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error al crear contexto: {}", e);
                    return;
                }
            };

            cr.scale(zoom_capped as f64, zoom_capped as f64);
            page.render(&cr);

            self.surface = Some(surface);
            self.texture = None;
            self.last_render_size = Some((render_w, render_h));
        }
    }

    pub fn next_page(&mut self) {
        if self.page_num < self.doc.n_pages() - 1 {
            self.page_num += 1;
            self.render_page();
        }
    }

    pub fn prev_page(&mut self) {
        if self.page_num > 0 {
            self.page_num -= 1;
            self.render_page();
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.25).min(4.0);
        self.render_page();
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.25).max(0.25);
        self.render_page();
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

    pub fn get_page(&self) -> i32 {
        self.page_num
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_page(&mut self, page: i32) {
        if page >= 0 && page < self.doc.n_pages() {
            self.page_num = page;
            self.render_page();
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 4.0);
        self.render_page();
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

    pub fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
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

        egui::ScrollArea::new([true, true]).show(ui, |ui: &mut egui::Ui| {
            if let Some((width, height, should_rebuild)) = self.get_render_info() {
                if let Some(surface) = &mut self.surface {
                    if let Ok(data) = surface.data() {
                        let mut bytes = data.to_vec();
                        // Cairo ARGB32 (ABGR) -> RGBA: swap R and B channels
                        for chunk in bytes.chunks_mut(4) {
                            chunk.swap(0, 2);
                        }
                        let image =
                            egui::ColorImage::from_rgba_unmultiplied([width, height], &bytes);

                        if should_rebuild {
                            self.texture =
                                Some(ctx.load_texture("page", image, Default::default()));
                        }

                        let size = egui::vec2(width as f32, height as f32);

                        ui.set_min_size(size);
                        if let Some(ref texture) = self.texture {
                            ui.add(egui::Image::new(texture).max_size(size));
                        }
                    }
                }
            }
        });

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

        go_back
    }
}
