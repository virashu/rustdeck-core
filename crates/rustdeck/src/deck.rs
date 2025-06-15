use indexmap::IndexMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::buttons::{DeckButton, DeckButtonUpdate, RenderedDeckButton, VariableRenderer};
use crate::config::{DeckButtonScreen, DeckConfig, DeckDimensionConfig};
use crate::constants::{DECK_ACTION_ID, DECK_ACTION_NAME, DECK_ACTION_PREFIX, PLUGIN_DIR};
use crate::models::PluginActionsData;
use crate::plugins::PluginStore;

mod config {
    use std::time::Duration;

    /// Update thread loop interval in millis
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(1000);
}

#[derive(Debug, serde::Serialize)]
pub struct DeckScreen {
    screen: String,
    buttons: Vec<RenderedDeckButton>,
}

pub struct Deck {
    config: DeckDimensionConfig,
    config_callback: Arc<dyn Fn(&DeckConfig) + Send + Sync + 'static>,
    current_screen_id: RwLock<String>,
    screens: IndexMap<String, RwLock<DeckButtonScreen>>,
    plugin_store: PluginStore,
    icons: HashMap<String, String>,
    #[allow(clippy::struct_field_names)]
    /// Actions of the deck itself
    deck_actions: PluginActionsData,
}

impl Deck {
    pub fn new(
        config: DeckConfig,
        config_callback: impl Fn(&DeckConfig) + Send + Sync + 'static,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_store = PluginStore::new(PLUGIN_DIR)?;

        Ok(Self {
            config: config.deck,
            config_callback: Arc::new(config_callback),
            current_screen_id: RwLock::new(String::from("default")),
            screens: config
                .screens
                .into_iter()
                .map(|s| (s.0, RwLock::new(s.1)))
                .collect(),
            plugin_store,
            icons: HashMap::from([("test_icon".into(), "icons/test_icon.png".into())]),
            deck_actions: PluginActionsData {
                id: String::from(DECK_ACTION_ID),
                name: String::from(DECK_ACTION_NAME),
                actions: vec![String::from("switch_screen")],
            },
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

    /// Handles click of a button at position (y, x)
    pub fn handle_click_at(&self, pos: (u32, u32)) -> Result<(), String> {
        let action = self
            .get_current_screen()
            .read()
            .get(&pos)
            .expect("Failed to get button at required index")
            .on_click_action
            .clone();

        match action {
            Some(id) if id.starts_with(DECK_ACTION_PREFIX) => {
                self.try_run_deck_action(id.strip_prefix(DECK_ACTION_PREFIX).unwrap())
            }
            Some(id) => self.plugin_store.try_run_action(id),
            None => Ok(()),
        }
    }

    /// Get `RwLock` of the currently selected button screen
    fn get_current_screen(&self) -> &RwLock<DeckButtonScreen> {
        self.screens
            .get(&self.current_screen_id.read().clone())
            .unwrap()
    }

    /// Get disk path of icon by its id
    pub fn get_icon<S>(&self, id: S) -> Option<&String>
    where
        S: AsRef<str>,
    {
        self.icons.get(id.as_ref())
    }

    //
    // Getters
    //
    pub fn get_dimensions_config(&self) -> DeckDimensionConfig {
        self.config.clone()
    }

    /// Get names of all available button screens
    pub fn get_available_screens(&self) -> Vec<String> {
        self.screens.keys().map(ToOwned::to_owned).collect()
    }

    /// Get a render of currently selected screen
    pub fn get_rendered_screen(&self) -> DeckScreen {
        let mut vars = VariableRenderer::new(&self.plugin_store);

        DeckScreen {
            screen: self.current_screen_id.read().clone(),
            buttons: self
                .get_current_screen()
                .read()
                .iter()
                .map(|(pos, b)| b.render(*pos, &mut vars))
                .collect(),
        }
    }

    /// Get (not rendered) button by position (y, x)
    pub fn get_raw_button(&self, pos: (u32, u32)) -> DeckButton {
        self.get_current_screen()
            .read()
            .get(&pos)
            .cloned()
            .unwrap_or_default()
    }

    /// Get names and values of all available variables
    pub fn get_all_variables(&self) -> HashMap<String, String> {
        self.plugin_store.get_all_variables()
    }

    /// Get names of all available actions
    pub fn get_all_actions_names(&self) -> Vec<String> {
        [
            self.deck_actions
                .actions
                .iter()
                .map(|a| format!("{DECK_ACTION_PREFIX}{a}"))
                .collect(),
            self.plugin_store.get_all_actions_names(),
        ]
        .concat()
    }

    /// Get all actions with plugin id and name info
    pub fn get_all_actions(&self) -> Vec<PluginActionsData> {
        let mut actions = self.plugin_store.get_all_actions();
        actions.insert(0, self.deck_actions.clone());
        actions
    }

    /// Change raw button properties (`template`, `on_click_action`, etc.)
    #[allow(clippy::significant_drop_tightening)] // Bro tweaking (false-positive)
    pub fn update_button(&self, pos: (u32, u32), update: DeckButtonUpdate) {
        {
            let mut lock = self.get_current_screen().write();
            let button;
            if let Some(b) = lock.get_mut(&pos) {
                button = b;
            } else {
                lock.insert(pos, DeckButton::default());
                button = lock.get_mut(&pos).unwrap();
            }

            button.template = update.template;
            button.on_click_action = update.on_click_action;
            button.icon = update.icon;
        }

        (self.config_callback)(&self.get_config());
    }

    pub fn delete_button(&self, pos: (u32, u32)) -> bool {
        let success = {
            let mut lock = self.get_current_screen().write();
            lock.remove(&pos).is_some()
        };
        if success {
            (self.config_callback)(&self.get_config());
        }
        success
    }

    pub fn switch_screen(&self, id: String) {
        if !self.screens.contains_key(&id) || *self.current_screen_id.read() == id {
            return;
        }

        *self.current_screen_id.write() = id;
    }

    pub fn get_config(&self) -> DeckConfig {
        DeckConfig {
            deck: self.config.clone(),
            screens: self
                .screens
                .iter()
                .map(|(id, screen)| (id.clone(), screen.read().clone()))
                .collect(),
        }
    }
}
