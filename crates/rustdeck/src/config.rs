use std::collections::HashMap;

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
pub type DeckScreens = HashMap<String, DeckButtonScreen>;

#[derive(Deserialize, Serialize)]
struct SerializedDeckButton {
    position: DeckButtonPos,
    style: DeckButtonStyle,
    template: String,
    on_click_action: Option<String>,
    icon: Option<String>,
}

impl SerializedDeckButton {
    pub fn to_deck_button(self) -> ((u32, u32), DeckButton) {
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
    fn to_deck_button_screen(self) -> (String, DeckButtonScreen) {
        (
            self.id,
            self.buttons
                .into_iter()
                .map(SerializedDeckButton::to_deck_button)
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
                .map(SerializedDeckButtonScreen::to_deck_button_screen)
                .collect(),
        }
    }
}

pub struct DeckConfig {
    pub deck: DeckDimensionConfig,
    pub screens: DeckScreens,
}

impl Default for DeckConfig {
    /// Create a base `default` named screen
    fn default() -> Self {
        Self {
            deck: DeckDimensionConfig::default(),
            screens: HashMap::from([("default".into(), HashMap::default())]),
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
