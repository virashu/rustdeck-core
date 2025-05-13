use std::{collections::HashMap, thread, time::Duration};

use crate::plugins::{self, Plugin};

mod config {
    pub const PLUGIN_DIR: &str = "./plugins";
}

pub struct DeckButton {}

pub struct Deck {
    buttons: HashMap<(usize, usize), DeckButton>,
    plugins: Vec<Plugin>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            buttons: HashMap::new(),
            plugins: plugins::load_plugins_at(std::path::Path::new(config::PLUGIN_DIR))?,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.plugins.iter_mut().for_each(|p| p.update());

            thread::sleep(Duration::from_millis(100));
        }
    }

    fn try_resolve_variable(&self, id: &str) -> Result<String, String> {
        let (plug_id, i) = id.split_once('.').ok_or("Wrong variable format")?;
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.id == plug_id)
            .ok_or(format!("Cannot find plugin: `{}`", plug_id))?;

        if !plugin.variables.contains(&i.to_string()) {
            return Err(format!("Plugin `{}` does not provide variable `{}`", plug_id, i));
        }

        Ok(plugin.get_variable(i.to_string()))
    }

    fn render_variable(&self, id: &str) -> String {
        match self.try_resolve_variable(id) {
            Ok(s) => s,
            Err(s) => s,
        }
    }

    fn try_run_action(&self, id: &str) -> Result<(), String> {
        let (plug_id, i) = id.split_once('.').ok_or("Wrong action format")?;
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.id == plug_id)
            .ok_or(format!("Cannot find plugin: `{}`", plug_id))?;

        if !plugin.actions.contains(&i.to_string()) {
            return Err(format!("Plugin `{}` does not provide action `{}`", plug_id, i));
        }

        plugin.run_action(i.to_string());

        Ok(())
    }
}
