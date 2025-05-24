use std::fs;
use std::path::Path;

use super::Plugin;

pub fn load_plugins_at(path: &Path) -> Result<Vec<Plugin>, Box<dyn std::error::Error>> {
    let plugins: Vec<_> = fs::read_dir(path)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            let filename = p.to_str().unwrap();
            let is_plugin = filename.ends_with(".deckplugin");

            if !is_plugin {
                tracing::warn!("Non-plugin found in 'plugins' directory: '{}'. Note that rustdeck plugins should have a `.deckplugin` extension.", filename);
            }

            is_plugin
        }).filter_map(|p| {
            match Plugin::try_load(&p) {
                Ok(plugin) => {
                    tracing::info!("Loaded plugin {:?}", p);
                    Some(plugin)
                }
                Err(e) => {
                    tracing::error!("Error loading plugin {:?}: {}", p, e);
                    None
                }
            }
        }).collect();

    tracing::info!("Loaded plugins ({})", plugins.len());

    Ok(plugins)
}
