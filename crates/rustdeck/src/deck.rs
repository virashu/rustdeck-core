use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::plugins::{self, Plugin};

mod config {
    use std::time::Duration;

    pub const PLUGIN_DIR: &str = "./plugins";

    /// Update thread loop interval in millis
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
}

pub struct DeckButton {}

pub struct Deck {
    buttons: HashMap<(usize, usize), DeckButton>,
    plugins: HashMap<String, Mutex<Plugin>>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let plugins = plugins::load_plugins_at(std::path::Path::new(config::PLUGIN_DIR))?;
        let plugins = plugins
            .into_iter()
            .map(|p| (p.id.clone(), Mutex::new(p)))
            .collect();

        Ok(Self {
            buttons: HashMap::new(),
            plugins,
        })
    }

    fn server_thread(&self) {
        tracing::debug!("Started render thread");

        loop {
            tracing::info!("{}", self.render_variable("plugin_test.counter"));

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn run(self) {
        let s = Arc::new(self);

        let c = s.clone();
        thread::spawn(move || {
            Self::server_thread(&c);
        });

        let mut inst = Instant::now();

        loop {
            if inst.elapsed() > config::UPDATE_INTERVAL {
                s.plugins.values().for_each(|p| p.lock().unwrap().update());

                s.try_run_action("plugin_test.increment");

                inst = Instant::now();
            }
        }
    }

    fn try_resolve_variable(&self, id: &str) -> Result<String, String> {
        let (plug_id, i) = id.split_once('.').ok_or("Wrong variable format")?;
        let plugin = self
            .plugins
            .get(plug_id)
            .ok_or(format!("Cannot find plugin: `{}`", plug_id))?
            .lock()
            .unwrap();

        if !plugin.variables.contains(&i.to_string()) {
            return Err(format!(
                "Plugin `{}` does not provide variable `{}`",
                plug_id, i
            ));
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
            .get(plug_id)
            .ok_or(format!("Cannot find plugin: `{}`", plug_id))?
            .lock()
            .unwrap();

        if !plugin.actions.contains(&i.to_string()) {
            return Err(format!(
                "Plugin `{}` does not provide action `{}`",
                plug_id, i
            ));
        }

        plugin.run_action(i.to_string());

        Ok(())
    }
}
