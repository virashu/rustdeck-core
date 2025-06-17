use std::{collections::HashMap, fs, path};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::buttons::{DeckButton, DeckButtonPos, DeckButtonStyle};

#[derive(Clone, Deserialize, Serialize)]
pub struct DeckDimensionConfig {
    pub cols: u32,
    pub rows: u32,
}

impl Default for DeckDimensionConfig {
    fn default() -> Self {
        Self { cols: 5, rows: 3 }
    }
}

pub type DeckButtonScreen = HashMap<(u32, u32), DeckButton>;
pub type DeckScreens = IndexMap<String, DeckButtonScreen>;

#[derive(Deserialize, Serialize)]
struct SerializedDeckButton {
    position: DeckButtonPos,
    style: DeckButtonStyle,
    template: String,
    on_click_action: Option<String>,
    icon: Option<String>,
}

impl SerializedDeckButton {
    pub fn into_deck_button(self) -> ((u32, u32), DeckButton) {
        (
            self.position.as_yx(),
            DeckButton {
                style: self.style,
                icon: self.icon,
                template: self.template,
                on_click_action: self.on_click_action,
            },
        )
    }

    pub fn from_deck_button(pos: (u32, u32), value: DeckButton) -> Self {
        Self {
            position: DeckButtonPos::from_yx(pos),
            icon: value.icon,
            on_click_action: value.on_click_action,
            style: value.style,
            template: value.template,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SerializedDeckButtonScreen {
    id: String,
    buttons: Vec<SerializedDeckButton>,
}

impl SerializedDeckButtonScreen {
    fn into_deck_button_screen(self) -> (String, DeckButtonScreen) {
        (
            self.id,
            self.buttons
                .into_iter()
                .map(SerializedDeckButton::into_deck_button)
                .collect(),
        )
    }

    fn from_deck_button_screen(id: String, buttons: DeckButtonScreen) -> Self {
        Self {
            id,
            buttons: buttons
                .into_iter()
                .map(|(pos, value)| SerializedDeckButton::from_deck_button(pos, value))
                .collect(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SerializedDeckConfig {
    deck: DeckDimensionConfig,
    screens: Vec<SerializedDeckButtonScreen>,
    icons: HashMap<String, String>,
}

impl From<&DeckConfig> for SerializedDeckConfig {
    fn from(value: &DeckConfig) -> Self {
        Self {
            deck: value.deck.clone(),
            screens: value
                .screens
                .clone()
                .into_iter()
                .map(|(id, buttons)| {
                    SerializedDeckButtonScreen::from_deck_button_screen(id, buttons)
                })
                .collect(),
            icons: value.icons.clone(),
        }
    }
}

impl From<SerializedDeckConfig> for DeckConfig {
    fn from(value: SerializedDeckConfig) -> Self {
        Self {
            deck: value.deck,
            screens: value
                .screens
                .into_iter()
                .map(SerializedDeckButtonScreen::into_deck_button_screen)
                .collect(),
            icons: value.icons,
        }
    }
}

pub struct DeckConfig {
    pub deck: DeckDimensionConfig,
    pub screens: DeckScreens,
    pub icons: HashMap<String, String>,
}

impl Default for DeckConfig {
    /// Create a base `default` named screen
    fn default() -> Self {
        Self {
            deck: DeckDimensionConfig::default(),
            screens: IndexMap::from([("default".into(), HashMap::default())]),
            icons: HashMap::default(),
        }
    }
}

impl Serialize for DeckConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        std::convert::Into::<SerializedDeckConfig>::into(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DeckConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(SerializedDeckConfig::deserialize(deserializer)?.into())
    }
}

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
                    .ok_or(std::io::Error::new(std::io::ErrorKind::Other, ""))
                    .map(|p| p.to_path_buf())
            })
            .unwrap_or(".".into());

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
