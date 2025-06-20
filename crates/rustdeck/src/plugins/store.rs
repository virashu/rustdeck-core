use std::{collections::HashMap, io, path::Path, sync::RwLock};

use crate::{
    buttons::RawDeckButtonAction,
    models::{
        PluginActionArgsData, PluginActionsGroupedData, PluginActionsUngroupedData, PluginData,
        PluginVariablesGroupedData, PluginVariablesUngroupedData,
    },
};

use super::{Plugin, load_plugins_at};

pub struct PluginStore {
    plugins: HashMap<String, RwLock<Plugin>>,
}

impl PluginStore {
    pub fn new<S>(path: S) -> Result<Self, io::Error>
    where
        S: AsRef<str>,
    {
        let plugins = load_plugins_at(Path::new(path.as_ref()))?;
        let plugins = plugins
            .into_iter()
            .map(|p| (p.id.clone(), RwLock::new(p)))
            .collect();

        Ok(Self { plugins })
    }

    pub fn update_all(&self) {
        self.plugins
            .values()
            .for_each(|p| p.write().unwrap().update());
    }

    pub fn try_resolve_variable<S>(&self, id: S) -> Result<String, String>
    where
        S: AsRef<str>,
    {
        let (plug_id, i) = id.as_ref().split_once('.').ok_or("Wrong variable format")?;
        let plugin = self
            .plugins
            .get(plug_id)
            .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?
            .read()
            .unwrap();

        if !plugin.variables.iter().any(|v| v.id == i) {
            return Err(format!(
                "Plugin `{plug_id}` does not provide variable `{i}`"
            ));
        }

        Ok(plugin.get_variable(i))
    }

    pub fn render_variable<S>(&self, id: S) -> String
    where
        S: AsRef<str>,
    {
        match self.try_resolve_variable(id) {
            Err(s) | Ok(s) => s,
        }
    }

    pub fn try_run_action(&self, act: &RawDeckButtonAction) -> Result<(), String> {
        let (plug_id, i) = act
            .id
            .split_once('.')
            .ok_or_else(|| format!("Wrong action format: `{}`", act.id))?;

        {
            let plugin = self
                .plugins
                .get(plug_id)
                .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?
                .read()
                .unwrap();

            if !plugin.actions.iter().any(|v| v.id == i) {
                return Err(format!("Plugin `{plug_id}` does not provide action `{i}`"));
            }

            plugin.run_action(i.to_string());
        }

        Ok(())
    }

    /// Get variables of a plugin
    fn get_variables_of(plugin: &Plugin) -> Vec<PluginVariablesUngroupedData> {
        plugin
            .variables
            .iter()
            .map(|var| PluginVariablesUngroupedData {
                id: format!("{}.{}", plugin.id, var.id),
                description: var.description.clone(),
                r#type: var.r#type,
            })
            .collect()
    }

    /// Get all variables of all plugins in the same vector
    pub fn get_all_variables_ungrouped(&self) -> Vec<PluginVariablesUngroupedData> {
        self.plugins
            .values()
            .flat_map(|p| Self::get_variables_of(&p.read().unwrap()))
            .collect()
    }

    /// Get all varibles grouped by plugin with plugin `id` and `name`
    ///
    /// Does not include plugins without variables
    pub fn get_all_variables_grouped(&self) -> Vec<PluginVariablesGroupedData> {
        self.plugins
            .values()
            .filter_map(|p| {
                let lock = p.read().unwrap();
                let variables = Self::get_variables_of(&lock);
                if variables.is_empty() {
                    None
                } else {
                    Some(PluginVariablesGroupedData {
                        id: lock.id.clone(),
                        name: lock.name.clone(),
                        variables,
                    })
                }
            })
            .collect()
    }

    /// Get actions of a plugin
    fn get_actions_of(plugin: &Plugin) -> Vec<PluginActionsUngroupedData> {
        plugin
            .actions
            .iter()
            .map(|act| PluginActionsUngroupedData {
                id: format!("{}.{}", plugin.id, act.id),
                name: act.name.clone(),
                description: act.description.clone(),
                args: act
                    .args
                    .iter()
                    .cloned()
                    .map(|a| PluginActionArgsData {
                        id: a.id,
                        description: a.description,
                        r#type: String::from(match a.r#type {
                            0 => "bool",
                            1 => "int",
                            2 => "float",
                            _ => "string",
                        }),
                    })
                    .collect(),
            })
            .collect()
    }

    /// Get all actions of all plugins in the same vector
    pub fn get_all_actions_ungrouped(&self) -> Vec<PluginActionsUngroupedData> {
        self.plugins
            .values()
            .flat_map(|p| Self::get_actions_of(&p.read().unwrap()))
            .collect()
    }

    /// Get all actions grouped by plugin with plugin `id` and `name`
    ///
    /// Does not include plugins without actions
    pub fn get_all_actions_grouped(&self) -> Vec<PluginActionsGroupedData> {
        self.plugins
            .values()
            .filter_map(|p| {
                let lock = p.read().unwrap();
                let actions = Self::get_actions_of(&lock);
                if actions.is_empty() {
                    None
                } else {
                    Some(PluginActionsGroupedData {
                        id: lock.id.clone(),
                        name: lock.name.clone(),
                        actions,
                    })
                }
            })
            .collect()
    }

    pub fn get_all_plugins(&self) -> Vec<PluginData> {
        self.plugins
            .values()
            .map(|p| {
                let lock = p.read().unwrap();
                PluginData {
                    id: lock.id.clone(),
                    name: lock.name.clone(),
                    description: lock.description.clone(),
                    variables: Self::get_variables_of(&lock),
                    actions: Self::get_actions_of(&lock),
                }
            })
            .collect()
    }
}
