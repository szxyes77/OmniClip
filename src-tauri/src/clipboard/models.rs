use crate::database::models::ClipboardRecord;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "files")]
    Files,
}

impl ToString for ClipboardType {
    fn to_string(&self) -> String {
        match self {
            ClipboardType::Text => "text".to_string(),
            ClipboardType::Image => "image".to_string(),
            ClipboardType::Files => "files".to_string(),
        }
    }
}

impl std::str::FromStr for ClipboardType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(ClipboardType::Text),
            "image" => Ok(ClipboardType::Image),
            "files" => Ok(ClipboardType::Files),
            _ => Err(format!("Unknown clipboard type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClipboardContent {
    pub record_type: ClipboardType,
    pub content: String,
    pub title: String,
    pub thumbnail_path: Option<String>,
}

impl ClipboardContent {
    pub fn from_text(text: &str) -> Self {
        let preview = text.lines().next().unwrap_or("");
        let title = if preview.len() > 100 {
            format!("{}...", &preview[..100])
        } else {
            preview.to_string()
        };

        Self {
            record_type: ClipboardType::Text,
            content: text.to_string(),
            title,
            thumbnail_path: None,
        }
    }

    pub fn from_image(bytes: &[u8], width: usize, height: usize) -> Option<Self> {
        let img = image::load_from_memory_with_format(
            bytes,
            image::ImageFormat::Png,
        ).ok()?;

        let thumbnail = if width > 300 {
            let scale = 300.0 / width as f32;
            let new_height = (height as f32 * scale) as u32;
            imageops::resize(&img, 300, new_height, imageops::FilterType::Lanczos3)
        } else {
            img.clone()
        };

        let temp_dir = std::env::temp_dir().join("omniclip_images");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).ok()?;
        }

        let filename = format!("{}.png", chrono::Utc::now().timestamp_millis());
        let original_path = temp_dir.join(&filename);

        if img.save(&original_path).is_err() {
            return None;
        }

        let mut thumb_bytes = Vec::new();
        thumbnail
            .write_to(&mut Cursor::new(&mut thumb_bytes), image::ImageFormat::Png)
            .ok()?;

        let thumbnail_base64 = data_encoding::BASE64.encode(&thumb_bytes);

        Some(Self {
            record_type: ClipboardType::Image,
            content: original_path.to_string_lossy().to_string(),
            title: format!("Image {}x{}", width, height),
            thumbnail_path: Some(thumbnail_base64),
        })
    }

    pub fn from_files(paths: Vec<String>) -> Self {
        let content = serde_json::to_string(&paths).unwrap_or_default();
        let title = if paths.len() == 1 {
            if let Some(name) = paths[0].split(std::path::MAIN_SEPARATOR).last() {
                format!("File: {}", name)
            } else {
                format!("File: {}", paths[0])
            }
        } else {
            format!("{} files", paths.len())
        };

        Self {
            record_type: ClipboardType::Files,
            content,
            title,
            thumbnail_path: None,
        }
    }
}
