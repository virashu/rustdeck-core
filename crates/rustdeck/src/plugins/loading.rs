use std::{fs, path::Path};

use super::Plugin;

pub fn load_plugins_at(path: &Path) -> Result<Vec<Plugin>, std::io::Error> {
    let plugins: Vec<_> = fs::read_dir(path)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter_map(|p| {
            match p.to_str() {
                Some(s) if s.ends_with(".deckplugin") => { Some(p) }
                Some(s) => {
                    tracing::warn!("Non-plugin found in 'plugins' directory: '{}'. Note that rustdeck plugins should have a `.deckplugin` extension.", s);
                    None
                }
                _ => None
            }
        }).filter_map(|p| {
            match Plugin::try_load(&p) {
                Ok(plugin) => {
                    tracing::info!("Loaded plugin {:?}", p);
                    Some(plugin)
                }
                Err(e) => {
                    tracing::warn!("Error loading plugin {:?}: {}", p, e);
                    None
                }
            }
        }).collect();

    tracing::info!("Loaded plugins ({})", plugins.len());

    Ok(plugins)
}
