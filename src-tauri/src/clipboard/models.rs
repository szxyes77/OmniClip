#[derive(Debug, Clone)]
pub enum RecordType {
    Text,
    Image,
    Files,
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RecordType::Text => write!(f, "text"),
            RecordType::Image => write!(f, "image"),
            RecordType::Files => write!(f, "files"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClipboardContent {
    pub record_type: RecordType,
    pub content: String,
    pub title: String,
    pub thumbnail_path: Option<String>,
}

impl ClipboardContent {
    pub fn from_text(text: &str) -> Self {
        let title = if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        };

        Self {
            record_type: RecordType::Text,
            content: text.to_string(),
            title,
            thumbnail_path: None,
        }
    }
}
