use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Mutex;

use super::{load_plugins_at, Plugin};

pub enum ActionError {
    WrongActionFormat(String),
    PluginNotFound(String),
    ActionNotFound(String),
}

pub struct PluginStore {
    plugins: HashMap<String, Mutex<Plugin>>,
}

impl PluginStore {
    pub fn new<S>(path: S) -> Result<Self, io::Error>
    where
        S: AsRef<str>,
    {
        let plugins = load_plugins_at(Path::new(path.as_ref()))?;
        let plugins = plugins
            .into_iter()
            .map(|p| (p.id.clone(), Mutex::new(p)))
            .collect();

        Ok(Self { plugins })
    }

    pub fn update_all(&self) {
        self.plugins
            .values()
            .for_each(|p| p.lock().unwrap().update());
    }

    pub fn try_resolve_variable<T>(&self, id: T) -> Result<String, String>
    where
        T: AsRef<str>,
    {
        let (plug_id, i) = id.as_ref().split_once('.').ok_or("Wrong variable format")?;
        let plugin = self
            .plugins
            .get(plug_id)
            .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?
            .lock()
            .unwrap();

        if !plugin.variables.contains(&i.to_string()) {
            return Err(format!(
                "Plugin `{plug_id}` does not provide variable `{i}`"
            ));
        }

        Ok(plugin.get_variable(i))
    }

    pub fn render_variable<T>(&self, id: T) -> String
    where
        T: AsRef<str>,
    {
        match self.try_resolve_variable(id) {
            Err(s) | Ok(s) => s,
        }
    }

    pub fn try_run_action(&self, id: &str) -> Result<(), String> {
        let (plug_id, i) = id
            .split_once('.')
            .ok_or_else(|| format!("Wrong action format: `{id}`"))?;

        {
            let plugin = self
                .plugins
                .get(plug_id)
                .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?
                .lock()
                .unwrap();

            if !plugin.actions.contains(&i.to_string()) {
                return Err(format!("Plugin `{plug_id}` does not provide action `{i}`"));
            }

            plugin.run_action(i.to_string());
        }

        Ok(())
    }

    pub fn get_all_variables(&self) -> HashMap<String, String> {
        let mut vars = HashMap::<String, String>::new();

        for (plugin_id, plugin) in &self.plugins {
            let lock = plugin.lock().unwrap();
            for var in &lock.variables {
                let var_id = format!("{plugin_id}.{var}");
                vars.insert(var_id.clone(), self.render_variable(var_id));
            }
        }

        vars
    }
}
