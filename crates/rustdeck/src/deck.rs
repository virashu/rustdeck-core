use std::{collections::HashMap, sync::Arc, time::Instant};

use indexmap::IndexMap;
use parking_lot::RwLock;

use crate::{
    buttons::{
        DeckButtonUpdate, RawDeckButton, RawDeckButtonAction, RenderedDeckButton, VariableRenderer,
    },
    config::{DeckButtonScreen, DeckConfig, DeckDimensionConfig, paths::PLUGINS},
    constants::{DECK_ACTION_ID, DECK_ACTION_NAME, DECK_ACTION_PREFIX},
    icon_store::IconStore,
    models::{
        PluginActionArgsData, PluginActionsGroupedData, PluginActionsUngroupedData, PluginData,
        PluginVariablesGroupedData, PluginVariablesUngroupedData,
    },
    plugins::PluginStore,
};

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
    config: RwLock<DeckDimensionConfig>,
    config_callback: Arc<dyn Fn(&DeckConfig) + Send + Sync + 'static>,
    current_screen_id: RwLock<String>,
    screens: RwLock<IndexMap<String, DeckButtonScreen>>,
    plugin_store: PluginStore,
    icon_store: IconStore,
    #[allow(clippy::struct_field_names)]
    /// Actions of the deck itself
    deck_actions: PluginActionsGroupedData,
}

impl Deck {
    pub fn new(
        config: DeckConfig,
        config_callback: impl Fn(&DeckConfig) + Send + Sync + 'static,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_store = PluginStore::new(&*PLUGINS)?;
        let icon_store = IconStore::from_config(config.icons);

        Ok(Self {
            config: RwLock::new(config.deck),
            config_callback: Arc::new(config_callback),
            current_screen_id: RwLock::new(String::from("default")),
            screens: RwLock::new(config.screens.into_iter().collect()),
            plugin_store,
            icon_store,
            deck_actions: PluginActionsGroupedData {
                id: String::from(DECK_ACTION_ID),
                name: String::from(DECK_ACTION_NAME),
                actions: vec![PluginActionsUngroupedData {
                    id: String::from("deck.switch_screen"),
                    name: String::from("Switch screen"),
                    description: String::new(),
                    args: vec![PluginActionArgsData {
                        id: String::from("screen"),
                        description: String::from("Screen to switch to"),
                        r#type: String::from("string"),
                    }],
                }],
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

    fn try_run_deck_action(&self, act: &RawDeckButtonAction) -> Result<(), String> {
        #[allow(clippy::single_match, reason = "TODO")]
        match act.id.as_str() {
            "deck.switch_screen" => {
                let arg = act.args.first().ok_or("Missing argument")?;
                self.current_screen_id.write().clone_from(arg);
            }
            _ => {}
        }

        Ok(())
    }

    /// Handles click of a button at position (y, x)
    pub fn handle_click_at(&self, pos: (u32, u32)) -> Result<(), String> {
        let action = self
            .screens
            .read()
            .get(self.current_screen_id.read().as_str())
            .unwrap()
            .get(&pos)
            .expect("Failed to get button at required index")
            .on_click_action
            .clone();

        match action {
            Some(act) if act.id.starts_with(DECK_ACTION_PREFIX) => self.try_run_deck_action(&act),
            Some(act) => self.plugin_store.try_run_action(&act),
            None => Ok(()),
        }
    }

    /// Get disk path of icon by its id
    pub fn get_icon<S>(&self, id: S) -> Option<&String>
    where
        S: AsRef<str>,
    {
        self.icon_store.get_icon(id)
    }

    //
    // Getters
    //
    pub fn get_dimensions_config(&self) -> DeckDimensionConfig {
        self.config.read().clone()
    }

    /// Get names of all available button screens
    pub fn get_available_screens(&self) -> Vec<String> {
        self.screens.read().keys().map(ToOwned::to_owned).collect()
    }

    /// Get a render of currently selected screen
    pub fn get_rendered_screen(&self) -> DeckScreen {
        let mut vars = VariableRenderer::new(&self.plugin_store);

        DeckScreen {
            screen: self.current_screen_id.read().clone(),
            buttons: self
                .screens
                .read()
                .get(self.current_screen_id.read().as_str())
                .unwrap()
                .iter()
                .map(|(pos, b)| b.render(*pos, &mut vars))
                .collect(),
        }
    }

    /// Get (not rendered) button by position (y, x)
    pub fn get_raw_button(&self, pos: (u32, u32)) -> RawDeckButton {
        self.screens
            .read()
            .get(self.current_screen_id.read().as_str())
            .unwrap()
            .get(&pos)
            .cloned()
            .unwrap_or_default()
    }

    /// Get names and values of all available variables
    pub fn get_all_variables_ungrouped(&self) -> Vec<PluginVariablesUngroupedData> {
        self.plugin_store.get_all_variables_ungrouped()
    }

    /// Get names and values of all available variables grouped by plugin id
    pub fn get_all_variables_grouped(&self) -> Vec<PluginVariablesGroupedData> {
        self.plugin_store.get_all_variables_grouped()
    }

    /// Get ids of all available actions
    pub fn get_all_actions_ungrouped(&self) -> Vec<PluginActionsUngroupedData> {
        [
            self.deck_actions
                .actions
                .clone()
                .into_iter()
                .map(|a| PluginActionsUngroupedData {
                    id: format!("{DECK_ACTION_PREFIX}{}", a.id),
                    ..a
                })
                .collect(),
            self.plugin_store.get_all_actions_ungrouped(),
        ]
        .concat()
    }

    /// Get all actions with plugin id and name info
    pub fn get_all_actions_grouped(&self) -> Vec<PluginActionsGroupedData> {
        let mut actions = self.plugin_store.get_all_actions_grouped();
        actions.insert(0, self.deck_actions.clone());
        actions
    }

    pub fn get_all_plugins(&self) -> Vec<PluginData> {
        self.plugin_store.get_all_plugins()
    }

    pub fn get_all_icons(&self) -> Vec<String> {
        self.icon_store.keys()
    }

    pub fn update_config(&self, update: DeckDimensionConfig) {
        *self.config.write() = update;
    }

    /// Change raw button properties (`template`, `on_click_action`, etc.)
    #[allow(clippy::significant_drop_tightening, reason = "false-positive")] // Bro tweaking
    pub fn update_button(&self, pos: (u32, u32), update: DeckButtonUpdate) {
        {
            let mut screens_lock = self.screens.write();
            let screen = screens_lock
                .get_mut(self.current_screen_id.read().as_str())
                .unwrap();

            let button;
            if let Some(b) = screen.get_mut(&pos) {
                button = b;
            } else {
                screen.insert(pos, RawDeckButton::default());
                button = screen.get_mut(&pos).unwrap();
            }

            button.template = update.template;
            button.on_click_action = update.on_click_action;
            button.icon = update.icon;
            button.style = update.style;
        }

        {
            self.save_config();
        }
    }

    #[allow(clippy::significant_drop_tightening, reason = "false-positive")]
    pub fn delete_button(&self, pos: (u32, u32)) -> bool {
        let success = {
            let mut screens_lock = self.screens.write();
            let screen = screens_lock
                .get_mut(self.current_screen_id.read().as_str())
                .unwrap();
            screen.remove(&pos).is_some()
        };
        if success {
            self.save_config();
        }
        success
    }

    pub fn switch_screen(&self, id: String) {
        if !self.screens.read().contains_key(&id) || *self.current_screen_id.read() == id {
            return;
        }

        *self.current_screen_id.write() = id;
    }

    pub fn new_screen(&self, id: String) -> Result<(), ()> {
        if self.screens.read().contains_key(&id) {
            return Err(());
        }

        {
            self.screens.write().insert(id, HashMap::new());
        }

        {
            self.save_config();
        }
        Ok(())
    }

    pub fn rename_screen(&self, old_id: &str, new_id: String) -> Result<(), ()> {
        if !self.screens.read().contains_key(old_id) || self.screens.read().contains_key(&new_id) {
            return Err(());
        }

        {
            let mut screens_lock = self.screens.write();
            let index = screens_lock.get_index_of(old_id).unwrap();
            let screen = screens_lock.swap_remove(old_id).unwrap();
            screens_lock.insert(new_id, screen);
            let last = screens_lock.len() - 1;
            screens_lock.swap_indices(index, last);
        }

        {
            self.save_config();
        }
        Ok(())
    }

    pub fn delete_screen(&self, id: &str) -> Result<(), ()> {
        if !self.screens.read().contains_key(id) {
            return Err(());
        }

        {
            self.screens.write().shift_remove(id);
        }

        {
            self.save_config();
        }
        Ok(())
    }

    #[allow(clippy::significant_drop_tightening, reason = "false-positive")]
    pub fn swap_buttons(&self, a: (u32, u32), b: (u32, u32)) {
        {
            let mut screens_lock = self.screens.write();
            let screen = screens_lock
                .get_mut(self.current_screen_id.read().as_str())
                .unwrap();

            let button_a = screen.remove(&a);

            if let Some(button) = button_a {
                let button_b = screen.insert(b, button);

                if let Some(button) = button_b {
                    screen.insert(a, button);
                }
            } else {
                let button_b = screen.remove(&b);

                if let Some(button) = button_b {
                    screen.insert(a, button);
                }
            }
        }

        {
            self.save_config();
        }
    }

    /// # Notes
    /// `screens` read lock
    pub fn get_config(&self) -> DeckConfig {
        DeckConfig {
            deck: self.config.read().clone(),
            screens: self.screens.read().clone(),
            icons: self.icon_store.to_config(),
        }
    }

    /// # Notes
    /// `screens` read lock
    fn save_config(&self) {
        (self.config_callback)(&self.get_config());
    }
}
