use eframe::egui;

pub struct FileBrowser {
    current_path: std::path::PathBuf,
    selected_file: Option<String>,
}

impl FileBrowser {
    pub fn new() -> Self {
        Self {
            current_path: dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")),
            selected_file: None,
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.selected_file = None;
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
                let is_pdf = name.to_lowercase().ends_with(".pdf");

                if is_dir || is_pdf {
                    entries.push(FileEntry { name, is_dir, path });
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
        ui.horizontal(|ui| {
            if ui.button("^").clicked() {
                self.go_up();
            }
            ui.label(&self.current_path.display().to_string());
        });

        ui.separator();

        let entries = self.get_entries();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for file in entries {
                let icon = if file.is_dir { "[DIR]" } else { "[PDF]" };
                let button = egui::Button::new(format!("{} {}", icon, file.name));

                if ui.add(button).clicked() {
                    if file.is_dir {
                        self.go_to(file.path);
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
}

