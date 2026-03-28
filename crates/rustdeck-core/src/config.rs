use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::buttons::{DeckButtonPos, DeckButtonStyle, RawDeckButton, RawDeckButtonAction};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeckDimensionConfig {
    pub cols: u32,
    pub rows: u32,
}

impl Default for DeckDimensionConfig {
    fn default() -> Self {
        Self { cols: 5, rows: 3 }
    }
}

pub type DeckButtonScreen = HashMap<(u32, u32), RawDeckButton>;
pub type DeckScreens = IndexMap<String, DeckButtonScreen>;

/// Raw deck button with position
#[derive(Deserialize, Serialize)]
struct SerializedDeckButton {
    position: DeckButtonPos,
    style: DeckButtonStyle,
    template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    on_click_action: Option<RawDeckButtonAction>,
}

impl SerializedDeckButton {
    pub fn into_deck_button(self) -> ((u32, u32), RawDeckButton) {
        (
            self.position.as_yx(),
            RawDeckButton {
                template: self.template,
                style: self.style,
                icon: self.icon,
                on_click_action: self.on_click_action,
            },
        )
    }

    pub fn from_deck_button(pos: (u32, u32), value: RawDeckButton) -> Self {
        Self {
            position: DeckButtonPos::from_yx(pos),
            template: value.template,
            style: value.style,
            icon: value.icon,
            on_click_action: value.on_click_action,
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
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default = "HashMap::default"
    )]
    plugins: HashMap<String, HashMap<String, String>>,
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
            plugins: value.plugins.clone(),
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
            plugins: value.plugins,
        }
    }
}

#[derive(Debug)]
pub struct DeckConfig {
    pub deck: DeckDimensionConfig,
    pub screens: DeckScreens,
    pub icons: HashMap<String, String>,
    pub plugins: HashMap<String, HashMap<String, String>>,
}

impl Default for DeckConfig {
    /// Create a base `default` named screen
    fn default() -> Self {
        Self {
            deck: DeckDimensionConfig::default(),
            screens: IndexMap::from([("default".into(), HashMap::default())]),
            icons: HashMap::default(),
            plugins: HashMap::default(),
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
