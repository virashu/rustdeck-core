use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::buttons::{ButtonAction, DeckButton};
use crate::plugins::PluginStore;

mod config {
    use std::time::Duration;

    pub const PLUGIN_DIR: &str = "./plugins";

    /// Update thread loop interval in millis
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
}

mod mock {
    use std::collections::HashMap;

    use crate::buttons::{ButtonAction, DeckButton, DeckButtonStyle, DeckButtonStyleTextAlign};

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
                        content: String::from("Counter: {plugin_test.counter}"),
                        on_click_action: ButtonAction::PluginAction(String::from("plugin_test.increment")),
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
                        content: String::from("Clear counter"),
                        on_click_action: ButtonAction::PluginAction(String::from("plugin_test.clear")),
                        icon: None,
                    },
                ),
                (
                    (1, 3),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        content: String::new(),
                        on_click_action: ButtonAction::None,
                        icon: Some("test_icon".into()),
                    },
                ),
                (
                    (1, 4),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        content: String::from("Switch to screen 2"),
                        on_click_action: ButtonAction::DeckAction(String::from("switch_screen:screen_2")),
                        icon: None,
                    },
                ),
                (
                    (2, 3),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        content: String::from(
                            "State: {rustdeck_media.state}\\nTitle: '{rustdeck_media.title}'\\nArtist: '{rustdeck_media.artist}'",
                        ),
                        on_click_action: ButtonAction::PluginAction(String::from("rustdeck_media.play_pause")),
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
                content: String::from("Switch to screen 1"),
                on_click_action: ButtonAction::DeckAction(String::from("switch_screen:default")),
                icon: None,
            },
        )])
    }
}

struct DeckConfig {
    pub cols: u32,
    pub rows: u32,
}

type ButtonScreen = HashMap<(u32, u32), DeckButton>;

pub struct Deck {
    config: DeckConfig,
    current_screen_id: Mutex<String>,
    screens: HashMap<String, ButtonScreen>,
    plugin_store: PluginStore,
    icons: HashMap<String, String>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_store = PluginStore::new(config::PLUGIN_DIR)?;

        Ok(Self {
            current_screen_id: String::from("default").into(),
            screens: HashMap::from([
                (String::from("default"), mock::mock_buttons_screen_1()),
                (String::from("screen_2"), mock::mock_buttons_screen_2()),
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

    pub fn serialize_config(&self) -> String {
        format!(
            r#"{{"cols": {}, "rows": {}}}"#,
            self.config.cols, self.config.rows
        )
    }

    pub fn serialize_buttons(&self) -> String {
        let buttons: Vec<String> = self
            .get_current_screen()
            .iter()
            .map(|(k, b)| b.serialize(k.to_owned(), &self.plugin_store))
            .collect();

        format!(
            r#"{{"screen": "{}", "buttons": [{}]}}"#,
            self.current_screen_id.lock().unwrap(),
            buttons.join(", ")
        )
    }

    fn try_run_deck_action(&self, id: &str) -> Result<(), String> {
        let (action, args_str) = id.split_once(':').ok_or("Wrong format")?;
        let args: Vec<&str> = args_str.split(';').collect();

        if action == "switch_screen" {
            *self.current_screen_id.lock().unwrap() = args[0].to_string();
        }

        Ok(())
    }

    pub fn handle_click_at(&self, y: u32, x: u32) -> Result<(), String> {
        let action = &self
            .get_current_screen()
            .get(&(y, x))
            .expect("Failed to get button at required index")
            .on_click_action;

        match action {
            ButtonAction::DeckAction(id) => self.try_run_deck_action(id),
            ButtonAction::PluginAction(id) => self.plugin_store.try_run_action(id),
            ButtonAction::None => Ok(()),
        }
    }

    fn get_current_screen(&self) -> &ButtonScreen {
        self.screens
            .get(&self.current_screen_id.lock().unwrap().clone())
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
}
