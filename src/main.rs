use eframe::egui;
use lector_pdf::{
    app_state::AppState,
    comic_viewer::ComicViewer,
    cbr_viewer::CbrViewer,
    file_browser::FileBrowser,
    pdf_viewer::PdfViewer,
    viewer::{detect_format, Format, Viewer},
};
use std::time::{Duration, Instant};

struct App {
    file_browser: FileBrowser,
    viewer: Option<Box<dyn Viewer>>,
    app_state: AppState,
    current_file: Option<String>,
    last_save_time: Instant,
    last_saved_page: Option<i32>,
    last_saved_zoom: Option<f32>,
    open_error: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            file_browser: FileBrowser::new(),
            viewer: None,
            app_state: AppState::new(),
            current_file: None,
            last_save_time: Instant::now(),
            last_saved_page: None,
            last_saved_zoom: None,
            open_error: None,
        }
    }

    fn create_viewer(file_path: &str) -> Option<Box<dyn Viewer>> {
        // Clean file:// prefix if present
        let clean_path = if file_path.starts_with("file://") {
            file_path.strip_prefix("file://").unwrap_or(file_path)
        } else {
            file_path
        };

        let format = detect_format(clean_path);

        match format {
            Some(Format::Pdf) => PdfViewer::new(clean_path).map(|v| Box::new(v) as Box<dyn Viewer>),
            Some(Format::Cbz) => {
                ComicViewer::new(clean_path).map(|v| Box::new(v) as Box<dyn Viewer>)
            }
            Some(Format::Cbr) => {
                CbrViewer::new(clean_path).map(|v| Box::new(v) as Box<dyn Viewer>)
            }
            None => None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected) = self.file_browser.selected_file() {
                let needs_load = match &self.current_file {
                    Some(current) => current != &selected,
                    None => true,
                };

                if needs_load && !selected.is_empty() {
                    self.open_error = None;

                    if let Some(mut viewer) = Self::create_viewer(&selected) {
                        if let Some(file_state) = self.app_state.get_file_state(&selected) {
                            viewer.set_page(file_state.page);
                            viewer.set_zoom(file_state.zoom);
                            self.last_saved_page = Some(file_state.page);
                            self.last_saved_zoom = Some(file_state.zoom);
                        }
                        self.viewer = Some(viewer);
                        self.current_file = Some(selected.to_string());
                    } else {
                        self.open_error = Some(
                            "Error al abrir el archivo. Puede estar corrupto o vacío.".to_string(),
                        );
                    }
                }

                if let Some(ref mut viewer) = self.viewer {
                    let current_page = viewer.current_page();
                    let current_zoom = viewer.get_zoom();

                    let page_changed = self.last_saved_page != Some(current_page);
                    let zoom_changed = self
                        .last_saved_zoom
                        .map(|z| (z - current_zoom).abs())
                        .unwrap_or(1.0)
                        > 0.01;

                    if page_changed || zoom_changed {
                        let should_save =
                            self.last_save_time.elapsed() > Duration::from_millis(500);
                        if should_save {
                            self.app_state.update_file(
                                selected.to_string(),
                                current_page,
                                current_zoom,
                            );
                            self.last_save_time = Instant::now();
                            self.last_saved_page = Some(current_page);
                            self.last_saved_zoom = Some(current_zoom);
                        }
                    }

                    let go_back = viewer.render(ctx, ui);

                    if go_back || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.viewer = None;
                        self.current_file = None;
                        self.file_browser.clear_selection();
                    }
                } else {
                    if let Some(ref err) = self.open_error {
                        ui.label(err);
                    } else {
                        ui.label("Error al cargar el archivo");
                    }
                    if ui.button("Volver").clicked() {
                        self.open_error = None;
                        self.file_browser.clear_selection();
                    }
                }
            } else {
                self.file_browser.render(ui);
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Lector PDF", options, Box::new(|_cc| Box::new(App::new())));
}
