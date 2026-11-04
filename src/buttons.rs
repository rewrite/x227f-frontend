//! Every button image, kept in memory and looked up by hash.

use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

use axum::body::Bytes;

pub struct Button {
    pub content_type: &'static str,
    pub data: Bytes,
}

pub struct Buttons(HashMap<String, Button>);

impl Buttons {
    /// Loads every `{hash}.{ext}` image in `dir`, skipping anything else.
    pub fn load(dir: impl AsRef<Path>) -> io::Result<Self> {
        let mut buttons = HashMap::new();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            let (Some(hash), Some(ext)) = (
                path.file_stem().and_then(|s| s.to_str()),
                path.extension().and_then(|s| s.to_str()),
            ) else {
                continue;
            };
            let Some(content_type) = content_type(ext) else {
                eprintln!("skipping {}: unknown image type", path.display());
                continue;
            };
            let data = fs::read(&path)?.into();
            buttons.insert(hash.to_owned(), Button { content_type, data });
        }
        Ok(Self(buttons))
    }

    pub fn get(&self, hash: &str) -> Option<&Button> {
        self.0.get(hash)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

fn content_type(ext: &str) -> Option<&'static str> {
    Some(match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        "svg" => "image/svg+xml",
        _ => return None,
    })
}
