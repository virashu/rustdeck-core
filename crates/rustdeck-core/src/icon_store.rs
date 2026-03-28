use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[cfg(feature = "icon_store_b64")]
use base64::{Engine, prelude::BASE64_STANDARD};

#[derive(thiserror::Error, Debug)]
pub enum IconStoreGetError {
    #[error("Icon not found")]
    NotFound,
    #[error("Failed to load icon: {0}")]
    IoError(std::io::Error),
}

pub struct IconStore {
    store_path: PathBuf,
    icons: HashMap<String, PathBuf>,
}

impl IconStore {
    pub fn from_config(path: impl AsRef<Path>, icons: HashMap<String, String>) -> Self {
        Self {
            store_path: path.as_ref().to_path_buf(),
            icons: icons
                .into_iter()
                .map(|(id, icon)| (id, PathBuf::from(icon)))
                .collect(),
        }
    }

    #[must_use]
    pub fn to_config(&self) -> HashMap<String, String> {
        self.icons
            .clone()
            .iter()
            .map(|(id, path)| (id.clone(), path.to_string_lossy().to_string()))
            .collect()
    }

    pub fn get_icon_path<S>(&self, id: S) -> Option<PathBuf>
    where
        S: AsRef<str>,
    {
        self.icons.get(id.as_ref()).map(|p| self.store_path.join(p))
    }

    /// Get a raw icon
    ///
    /// # Errors
    /// Error is returned if icon is not found or cannot be read
    pub fn get_icon_raw<S>(&self, id: S) -> Result<Vec<u8>, IconStoreGetError>
    where
        S: AsRef<str>,
    {
        let icon_path = self.get_icon_path(id).ok_or(IconStoreGetError::NotFound)?;
        std::fs::read(&icon_path)
            .inspect_err(|e| tracing::warn!("Failed to read registered image ({icon_path:?}): {e}"))
            .map_err(IconStoreGetError::IoError)
    }

    /// Get a base64-encoded icon
    ///
    /// # Errors
    /// Error is returned if icon is not found or cannot be read
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

    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.icons.keys().cloned().collect()
    }
}
