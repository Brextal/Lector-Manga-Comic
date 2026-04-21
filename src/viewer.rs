use eframe::egui;

pub trait Viewer {
    fn next_page(&mut self);
    fn prev_page(&mut self);
    fn set_page(&mut self, page: i32);
    fn current_page(&self) -> i32;
    fn total_pages(&self) -> i32;
    fn zoom_in(&mut self);
    fn zoom_out(&mut self);
    fn set_zoom(&mut self, zoom: f32);
    fn get_zoom(&self) -> f32;
    fn zoom_percent(&self) -> i32;

    fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> bool;
    fn get_file_path(&self) -> &str;
    fn get_file_name(&self) -> &str;
}

pub fn detect_format(file_path: &str) -> Option<Format> {
    let lower = file_path.to_lowercase();
    if lower.ends_with(".pdf") {
        Some(Format::Pdf)
    } else if lower.ends_with(".cbz") || lower.ends_with(".zip") {
        Some(Format::Cbz)
    } else if lower.ends_with(".cbr") || lower.ends_with(".rar") {
        Some(Format::Cbr)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Pdf,
    Cbz,
    Cbr,
}
