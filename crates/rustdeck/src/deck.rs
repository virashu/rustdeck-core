use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::time::Instant;

use crate::buttons::{DeckButton, DeckButtonPos, DeckButtonUpdate, RenderedDeckButton};
use crate::config::DeckConfig;
use crate::plugins::PluginStore;

mod config {
    use std::time::Duration;

    pub const PLUGIN_DIR: &str = "./plugins";

    /// Update thread loop interval in millis
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
}

mod mock {
    use std::collections::HashMap;

    use crate::buttons::{DeckButton, DeckButtonStyle, DeckButtonStyleTextAlign};

    use super::DeckConfig;

    pub const fn mock_config() -> DeckConfig {
        DeckConfig { cols: 5, rows: 3 }
    }

    pub fn mock_buttons_screen_1() -> HashMap<(u32, u32), DeckButton> {
        HashMap::from([
                (
                    (1, 1),
                    DeckButton {
                        style: DeckButtonStyle {
                            text_align: DeckButtonStyleTextAlign::Right,
                            ..Default::default()
                        },
                        template: String::from("Counter: {plugin_test.counter}"),
                        on_click_action: Some(String::from("plugin_test.increment")),
                        icon: None,
                    },
                ),
                (
                    (1, 2),
                    DeckButton {
                        style: DeckButtonStyle {
                            text_align: DeckButtonStyleTextAlign::Left,
                            ..Default::default()
                        },
                        template: String::from("Clear counter"),
                        on_click_action: Some(String::from("plugin_test.clear")),
                        icon: None,
                    },
                ),
                (
                    (1, 3),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        template: String::new(),
                        on_click_action: None,
                        icon: Some("test_icon".into()),
                    },
                ),
                (
                    (1, 4),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        template: String::from("Switch to screen 2"),
                        on_click_action: Some(String::from("deck.switch_screen:screen_2")),
                        icon: None,
                    },
                ),
                (
                    (2, 3),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        template: String::from(
                            "State: {rustdeck_media.state}\\nTitle: '{rustdeck_media.title}'\\nArtist: '{rustdeck_media.artist}'",
                        ),
                        on_click_action: Some(String::from("rustdeck_media.play_pause")),
                        icon: None,
                    },
                ),
            ])
    }

    pub fn mock_buttons_screen_2() -> HashMap<(u32, u32), DeckButton> {
        HashMap::from([(
            (1, 1),
            DeckButton {
                style: DeckButtonStyle::default(),
                template: String::from("Switch to screen 1"),
                on_click_action: Some(String::from("deck.switch_screen:default")),
                icon: None,
            },
        )])
    }
}

type ButtonScreen = HashMap<(u32, u32), DeckButton>;

#[derive(Debug, serde::Serialize)]
pub struct DeckScreen {
    screen: String,
    buttons: Vec<RenderedDeckButton>,
}

pub struct Deck {
    config: DeckConfig,
    current_screen_id: RwLock<String>,
    screens: HashMap<String, RwLock<ButtonScreen>>,
    plugin_store: PluginStore,
    icons: HashMap<String, String>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_store = PluginStore::new(config::PLUGIN_DIR)?;

        Ok(Self {
            current_screen_id: String::from("default").into(),
            screens: HashMap::from([
                (
                    String::from("default"),
                    mock::mock_buttons_screen_1().into(),
                ),
                (
                    String::from("screen_2"),
                    mock::mock_buttons_screen_2().into(),
                ),
            ]),
            plugin_store,
            config: mock::mock_config(),
            icons: HashMap::from([("test_icon".into(), "icons/test_icon.png".into())]),
        })
    }

    pub fn run(&self) {
        let mut inst = Instant::now();

        loop {
            if inst.elapsed() > config::UPDATE_INTERVAL {
                self.plugin_store.update_all();

                inst = Instant::now();
            }
        }
    }

    fn try_run_deck_action<S>(&self, id: S) -> Result<(), String>
    where
        S: AsRef<str>,
    {
        let (action, args_str) = id.as_ref().split_once(':').ok_or("Wrong format")?;
        let args: Vec<&str> = args_str.split(';').collect();

        if action == "switch_screen" {
            *self.current_screen_id.write() = args[0].to_string();
        }

        Ok(())
    }

    pub fn handle_click_at(&self, pos: (u32, u32)) -> Result<(), String> {
        let action = self
            .get_current_screen()
            .read()
            .get(&pos)
            .expect("Failed to get button at required index")
            .on_click_action
            .clone();

        match action {
            Some(id) if id.starts_with("deck.") => {
                self.try_run_deck_action(id.strip_prefix("deck.").unwrap())
            }
            Some(id) => self.plugin_store.try_run_action(id),
            None => Ok(()),
        }
    }

    fn get_current_screen(&self) -> &RwLock<ButtonScreen> {
        self.screens
            .get(&self.current_screen_id.read().clone())
            .unwrap()
    }

    pub fn get_icon<S>(&self, id: S) -> Option<&String>
    where
        S: AsRef<str>,
    {
        self.icons.get(id.as_ref())
    }

    fn get_available_screens(&self) -> Vec<String> {
        self.screens.keys().map(ToOwned::to_owned).collect()
    }

    //
    // Getters
    //
    pub fn get_config(&self) -> DeckConfig {
        self.config.clone()
    }

    pub fn get_rendered_screen(&self) -> DeckScreen {
        DeckScreen {
            screen: self.current_screen_id.read().clone(),
            buttons: self
                .get_current_screen()
                .read()
                .iter()
                .map(|(pos, b)| b.render(*pos, &self.plugin_store))
                .collect(),
        }
    }

    pub fn get_raw_button(&self, pos: (u32, u32)) -> DeckButton {
        self.get_current_screen()
            .read()
            .get(&pos)
            .cloned()
            .unwrap_or_default()
    }

    //
    // Serialization
    //
    pub fn serialize_config(&self) -> String {
        format!(
            r#"{{"cols": {}, "rows": {}}}"#,
            self.config.cols, self.config.rows
        )
    }

    pub fn serialize_buttons(&self) -> String {
        let buttons: Vec<String> = self
            .get_current_screen()
            .read()
            .iter()
            .map(|(k, b)| b.serialize(k.to_owned(), &self.plugin_store))
            .collect();

        format!(
            r#"{{"screen": "{}", "buttons": [{}]}}"#,
            self.current_screen_id.read(),
            buttons.join(", ")
        )
    }

    pub fn serialize_actions(&self) -> String {
        format!(
            "[{}]",
            self.plugin_store
                .get_all_actions_names()
                .iter()
                .map(|s| format!(r#""{s}""#))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn serialize_variables(&self) -> String {
        format!(
            "[{}]",
            self.plugin_store
                .get_all_variables()
                .iter()
                .map(|(k, v)| format!(r#""{k}": "{v}""#))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn serialize_button_with_fallback(&self, pos: (u32, u32)) -> String {
        self.get_current_screen().read().get(&pos).map_or_else(
            || DeckButton::default().serialize(pos, &self.plugin_store),
            |b| b.serialize(pos, &self.plugin_store),
        )
    }

    pub fn update_button(&self, pos: (u32, u32), update: DeckButtonUpdate) {
        let mut lock = self.get_current_screen().write();
        let button = lock.get_mut(&pos).unwrap();

        button.template = update.template;
        button.on_click_action = update.on_click_action;
    }
}
