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

    fn take_page_input(&mut self) -> String;
    fn set_error_message(&mut self, msg: Option<String>);
}

pub fn render_navigation_bar(ui: &mut egui::Ui, viewer: &mut dyn Viewer) -> (bool, String) {
    let mut go_back = false;
    let mut page_input_text = viewer.take_page_input();

    ui.horizontal(|ui| {
        if ui.button("Abrir archivo").clicked() {
            go_back = true;
        }
        ui.add_space(10.0);

        if ui.button("Ant.").clicked() {
            viewer.prev_page();
        }
        ui.label(format!(
            "Página {} / {}",
            viewer.current_page() + 1,
            viewer.total_pages()
        ));

        let response = ui.add(
            egui::TextEdit::singleline(&mut page_input_text)
                .desired_width(40.0)
                .hint_text("Ir a...")
        );
        
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
        if response.lost_focus() && enter_pressed {
            let input = page_input_text.trim().to_string();
            if !input.is_empty() {
                if let Ok(p) = input.parse::<i32>() {
                    if p < 1 {
                        viewer.set_error_message(Some("La página debe ser >= 1".to_string()));
                    } else {
                        let target = p - 1;
                        if target >= 0 && target < viewer.total_pages() {
                            viewer.set_page(target);
                            viewer.set_error_message(None);
                            page_input_text.clear();
                        } else {
                            viewer.set_error_message(Some(format!("Página {} fuera de rango", p)));
                            page_input_text.clear();
                        }
                    }
                } else {
                    viewer.set_error_message(Some("Entrada inválida".to_string()));
                    page_input_text.clear();
                }
            }
        }

        if ui.button("Ir").clicked() {
            let input = page_input_text.trim().to_string();
            if !input.is_empty() {
                if let Ok(p) = input.parse::<i32>() {
                    if p < 1 {
                        viewer.set_error_message(Some("La página debe ser >= 1".to_string()));
                    } else {
                        let target = p - 1;
                        if target >= 0 && target < viewer.total_pages() {
                            viewer.set_page(target);
                            viewer.set_error_message(None);
                            page_input_text.clear();
                        } else {
                            viewer.set_error_message(Some(format!("Página {} fuera de rango", p)));
                            page_input_text.clear();
                        }
                    }
                } else {
                    viewer.set_error_message(Some("Entrada inválida".to_string()));
                    page_input_text.clear();
                }
            }
        }

        if ui.button("Sig.").clicked() {
            viewer.next_page();
        }
        ui.add_space(20.0);

        if ui.button("-").clicked() {
            viewer.zoom_out();
        }
        ui.label(format!("{}%", viewer.zoom_percent()));
        if ui.button("+").clicked() {
            viewer.zoom_in();
        }
    });

    (go_back, page_input_text)
}

pub fn handle_keyboard_shortcuts(ui: &egui::Ui, viewer: &mut dyn Viewer) -> bool {
    let mut go_back = false;

    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
        viewer.prev_page();
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
        viewer.next_page();
    }
    if ui.input(|i| i.key_pressed(egui::Key::Plus))
        || ui.input(|i| i.key_pressed(egui::Key::Equals))
    {
        viewer.zoom_in();
    }
    if ui.input(|i| i.key_pressed(egui::Key::Minus)) {
        viewer.zoom_out();
    }
    if ui.input(|i| i.key_pressed(egui::Key::Q)) {
        go_back = true;
    }

    go_back
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
