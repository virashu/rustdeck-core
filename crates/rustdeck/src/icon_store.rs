use std::collections::HashMap;

#[cfg(feature = "icon_store_b64")]
use base64::{Engine, prelude::BASE64_STANDARD};

pub enum IconStoreGetError {
    NotFound,
    #[allow(dead_code)]
    IoError(std::io::Error),
}

pub struct IconStore {
    icons: HashMap<String, String>,
}

impl IconStore {
    pub const fn from_config(icons: HashMap<String, String>) -> Self {
        Self { icons }
    }

    pub fn to_config(&self) -> HashMap<String, String> {
        self.icons.clone()
    }

    pub fn get_icon_path<S>(&self, id: S) -> Option<String>
    where
        S: AsRef<str>,
    {
        self.icons
            .get(id.as_ref())
            .map(|p| format!("{}/{p}", &*crate::config::paths::ICONS))
    }

    pub fn get_icon_raw<S>(&self, id: S) -> Result<Vec<u8>, IconStoreGetError>
    where
        S: AsRef<str>,
    {
        let icon_path = self.get_icon_path(id).ok_or(IconStoreGetError::NotFound)?;
        std::fs::read(icon_path)
            .inspect_err(|e| tracing::warn!("Failed to read registered image: {e}"))
            .map_err(IconStoreGetError::IoError)
    }

    #[cfg(feature = "icon_store_b64")]
    pub fn get_icon_b64<S>(&self, id: S) -> Result<String, IconStoreGetError>
    where
        S: AsRef<str>,
    {
        let icon_raw = self.get_icon_raw(id)?;
        Ok(BASE64_STANDARD.encode(icon_raw))
    }

    // TODO
    #[allow(
        dead_code,
        clippy::unused_self,
        clippy::needless_pass_by_ref_mut,
        reason = "TODO"
    )]
    pub fn add_icon(&mut self) {}

    pub fn keys(&self) -> Vec<String> {
        self.icons.keys().cloned().collect()
    }
}
