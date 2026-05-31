use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub max_history_count: usize,
    pub auto_cleanup_days: usize,
    pub keep_starred: bool,
    pub global_shortcut: String,
    pub ignore_apps: Vec<String>,
    pub theme: String,
    #[serde(default)]
    pub master_password_hash: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_history_count: 1000,
            auto_cleanup_days: 30,
            keep_starred: true,
            global_shortcut: "CmdOrCtrl+Shift+V".to_string(),
            ignore_apps: vec![],
            theme: "system".to_string(),
            master_password_hash: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub max_history_count: Option<usize>,
    pub auto_cleanup_days: Option<usize>,
    pub keep_starred: Option<bool>,
    pub global_shortcut: Option<String>,
    pub theme: Option<String>,
    pub master_password_hash: Option<String>,
    pub ignore_apps: Option<Vec<String>>,
}
