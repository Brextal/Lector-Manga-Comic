use eframe::egui;

pub struct FileBrowser {
    current_path: std::path::PathBuf,
    selected_file: Option<String>,
    path_input: String,
    highlighted_index: Option<usize>,
}

impl FileBrowser {
    pub fn new() -> Self {
        Self {
            current_path: dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")),
            selected_file: None,
            path_input: String::new(),
            highlighted_index: None,
        }
    }

    pub fn go_up(&mut self) {
        let limit = std::path::Path::new("/home/brextal/");
        if let Some(parent) = self.current_path.parent() {
            // Check if parent is still within the limit
            if parent.starts_with(limit) || parent == std::path::Path::new("/") {
                self.current_path = parent.to_path_buf();
                self.selected_file = None;
            }
        }
    }

    pub fn go_to(&mut self, path: std::path::PathBuf) {
        if path.is_dir() {
            self.current_path = path;
            self.selected_file = None;
        }
    }

    pub fn selected_file(&self) -> Option<&str> {
        self.selected_file.as_deref()
    }

    pub fn select_file(&mut self, file: String) {
        self.selected_file = Some(file);
    }

    pub fn clear_selection(&mut self) {
        self.selected_file = None;
        self.highlighted_index = None;
    }

    pub fn go_to_path(&mut self, path: String) {
        let path_buf = std::path::PathBuf::from(&path);
        if path_buf.is_dir() {
            self.current_path = path_buf;
            self.selected_file = None;
        } else if path_buf.is_file() {
            self.selected_file = Some(path);
        }
    }

    fn get_entries(&self) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        if let Ok(dir) = std::fs::read_dir(&self.current_path) {
            for entry in dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let lower = name.to_lowercase();
                let is_pdf = lower.ends_with(".pdf");
                let is_cbz = lower.ends_with(".cbz") || lower.ends_with(".zip");
                let is_cbr = lower.ends_with(".cbr") || lower.ends_with(".rar");

                if is_dir || is_pdf || is_cbz || is_cbr {
                    let file_type = if is_dir {
                        "DIR"
                    } else if is_pdf {
                        "PDF"
                    } else if is_cbz {
                        "CBZ"
                    } else {
                        "CBR"
                    };
                    entries.push(FileEntry {
                        name,
                        is_dir,
                        path,
                        file_type: file_type.to_string(),
                    });
                }
            }
        }

        entries.sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        entries
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Handle backspace to go up a directory
        if ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
            self.go_up();
        }

        ui.horizontal(|ui| {
            if ui.button("^").clicked() {
                self.go_up();
            }
            ui.label(&self.current_path.display().to_string());
        });

        ui.horizontal(|ui| {
            let response =
                ui.add(egui::TextEdit::singleline(&mut self.path_input).desired_width(300.0));
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.path_input.is_empty() {
                    self.go_to_path(self.path_input.clone());
                    self.path_input.clear();
                }
            }
            if ui.button("Ir").clicked() {
                if !self.path_input.is_empty() {
                    self.go_to_path(self.path_input.clone());
                    self.path_input.clear();
                }
            }
        });

        ui.separator();

        let entries = self.get_entries();
        let mut triggered_click = false;

        // Handle keyboard navigation
        if !entries.is_empty() {
            let max_index = entries.len() - 1;

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.highlighted_index = match self.highlighted_index {
                    Some(idx) if idx < max_index => Some(idx + 1),
                    Some(idx) => Some(idx),
                    None => Some(0),
                };
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.highlighted_index = match self.highlighted_index {
                    Some(idx) if idx > 0 => Some(idx - 1),
                    Some(idx) => Some(idx),
                    None => Some(0),
                };
            }

            // Handle Enter key for selected entry
            if let Some(idx) = self.highlighted_index {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(entry) = entries.get(idx) {
                        if entry.is_dir {
                            self.go_to(entry.path.clone());
                            self.highlighted_index = None;
                        } else {
                            self.select_file(entry.path.to_string_lossy().to_string());
                            triggered_click = true;
                        }
                    }
                }
            }
        } else {
            self.highlighted_index = None;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, file) in entries.iter().enumerate() {
                let icon = format!("[{}]", file.file_type);
                let is_highlighted = self.highlighted_index == Some(i);

                let mut button = egui::Button::new(format!("{} {}", icon, file.name));

                // Highlight the current entry
                if is_highlighted {
                    button = button.fill(egui::Color32::from_gray(80));
                }

                let clicked = ui.add(button).clicked();

                if clicked || (is_highlighted && triggered_click) {
                    if file.is_dir {
                        self.go_to(file.path.clone());
                        self.highlighted_index = None;
                    } else {
                        self.select_file(file.path.to_string_lossy().to_string());
                    }
                }
            }
        });
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub path: std::path::PathBuf,
    pub file_type: String,
}
