use std::{fs, path};

use rustdeck_core::config::DeckConfig;

pub mod paths {
    #[cfg(feature = "portable")]
    use std::env;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    fn get_root() -> PathBuf {
        #[cfg(feature = "portable")]
        return env::current_exe()
            .and_then(|p| {
                p.parent()
                    .ok_or_else(|| std::io::Error::other(""))
                    .map(std::path::Path::to_path_buf)
            })
            .unwrap_or_else(|_| ".".into());

        #[cfg(not(feature = "portable"))]
        return ".".into();
    }

    pub static ROOT_PATH: LazyLock<PathBuf> = LazyLock::new(get_root);

    fn get_config() -> String {
        ROOT_PATH.join("config.json").to_string_lossy().to_string()
    }

    fn get_plugins() -> String {
        ROOT_PATH.join("plugins").to_string_lossy().to_string()
    }

    fn get_icons() -> String {
        ROOT_PATH.join("icons").to_string_lossy().to_string()
    }

    pub static ICONS: LazyLock<String> = LazyLock::new(get_icons);
    pub static PLUGINS: LazyLock<String> = LazyLock::new(get_plugins);
    pub static CONFIG: LazyLock<String> = LazyLock::new(get_config);
}

pub fn load_config() -> DeckConfig {
    if !path::Path::new(&*paths::CONFIG).exists() {
        let config = DeckConfig::default();
        let config_ser = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&*paths::CONFIG, config_ser).expect("Failed to write default config");

        return config;
    }

    let config_ser = fs::read(&*paths::CONFIG).expect("Failed to read config");
    serde_json::from_slice(&config_ser).expect("Failed to deserialize config")
}

pub fn save_config(config: &DeckConfig) {
    let config_ser = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&*paths::CONFIG, config_ser).expect("Failed to write config");
}
