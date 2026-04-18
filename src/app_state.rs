use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct FileState {
    pub page: i32,
    pub zoom: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct AppState {
    pub file_states: HashMap<String, FileState>,
    pub last_opened: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let state_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lector-pdf");

        if state_dir.components().any(|c| c.as_os_str() == "..") {
            return Self::default();
        }

        if fs::create_dir_all(&state_dir).is_err() {
            return Self::default();
        }

        let state_file = state_dir.join("state.json");

        let mut state = Self::default();

        if let Ok(content) = fs::read_to_string(&state_file) {
            if let Ok(loaded) = serde_json::from_str::<AppState>(&content) {
                state = loaded;
            }
        }

        state
    }

    pub fn save(&self) {
        let state_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lector-pdf");

        let state_file = state_dir.join("state.json");

        if let Ok(content) = serde_json::to_string_pretty(self) {
            fs::write(&state_file, content).ok();
        }
    }

    pub fn get_file_state(&self, file: &str) -> Option<&FileState> {
        if file.contains("..") || (file.starts_with('/') && !file.starts_with("/home")) {
            return None;
        }
        self.file_states.get(file)
    }

    pub fn update_file(&mut self, file: String, page: i32, zoom: f32) {
        let page = page.max(0);
        let zoom = zoom.clamp(0.25, 4.0);

        self.file_states
            .insert(file.clone(), FileState { page, zoom });
        self.last_opened = Some(file);
        self.save();
    }

    pub fn get_last_opened(&self) -> Option<&str> {
        self.last_opened.as_deref()
    }
}
