mod plugin_wrapper;

use std::fs;
use std::path::Path;

pub use plugin_wrapper::Plugin;

pub fn load_plugins_at(path: &Path) -> Result<Vec<Plugin>, Box<dyn std::error::Error>> {
    let mut plugins = Vec::new();

    let dir = fs::read_dir(path)?;
    let entries = dir.flatten();
    let paths = entries.map(|e| e.path()).collect::<Vec<_>>();
    let libs = &paths
        .iter()
        .filter(|p| p.is_file())
        .filter(|p| {
            let filename = p.to_str().unwrap();
            let is_plugin = filename.ends_with(".deckplugin");

            if !is_plugin {
                tracing::warn!(r"Non-plugin found in 'plugins' directory: '{}'. Note that rustdeck plugins should have a `.deckplugin` extension.", filename);
            }

            is_plugin
        })
        .collect::<Vec<_>>();

    for path in libs {
        match Plugin::try_load(path) {
            Ok(plugin) => {
                plugins.push(plugin);
            }
            Err(e) => {
                tracing::error!("Error loading {:?}:\n -> {}", path, e);
            }
        }
    }

    tracing::info!("Loaded plugins ({})", plugins.len());

    Ok(plugins)
}
