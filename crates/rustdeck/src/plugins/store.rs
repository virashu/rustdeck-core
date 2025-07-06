use std::{collections::HashMap, io, path::Path, sync::Arc};

use parking_lot::RwLock;

use crate::{
    buttons::RawDeckButtonAction,
    models::{
        PluginActionArgsData, PluginActionsGroupedData, PluginActionsUngroupedData, PluginData,
        PluginVariablesGroupedData, PluginVariablesUngroupedData,
    },
    plugins::error::ActionError,
};

use super::{Plugin, load_plugins_at};

pub struct PluginStore {
    plugins: RwLock<HashMap<String, RwLock<Plugin>>>,
    plugins_uninit: Arc<RwLock<Vec<Plugin>>>,
}

impl PluginStore {
    pub fn new<S>(path: S) -> Result<Self, io::Error>
    where
        S: AsRef<str>,
    {
        tracing::info!("Loading plugins...");

        let plugins = load_plugins_at(Path::new(path.as_ref()))?;
        let plugins = RwLock::new(
            plugins
                .into_iter()
                .map(|p| (p.id.clone(), RwLock::new(p)))
                .collect(),
        );

        Ok(Self {
            plugins,
            plugins_uninit: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn init_all(&self) {
        tracing::info!("Initializing plugins...");

        let mut uninit = Vec::new();

        {
            self.plugins.read().values().for_each(|p| {
                let mut lock = p.write();
                _ = lock
                    .init()
                    .inspect(|()| tracing::info!("Initialized plugin {:?}", lock.id))
                    .inspect_err(|e| {
                        tracing::warn!("Failed to initialize plugin {:?}: {}", lock.id, e);
                        uninit.push(lock.id.clone());
                    });
            });

            tracing::info!("Initialized plugins");
        }

        {
            let mut plugins = self.plugins.write();
            let mut uninit_vec = self.plugins_uninit.write();

            for id in uninit {
                uninit_vec.push(plugins.remove(&id).unwrap().into_inner());
            }
        }
    }

    pub fn update_all(&self) {
        self.plugins
            .read()
            .values()
            .for_each(|p| p.write().update());
    }

    pub fn try_resolve_variable<S>(&self, id: S) -> Result<String, String>
    where
        S: AsRef<str>,
    {
        let (plug_id, i) = id.as_ref().split_once('.').ok_or("Wrong variable format")?;
        let plugins = self.plugins.read();
        let plugin = plugins
            .get(plug_id)
            .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?
            .read();

        if !plugin.variables.iter().any(|v| v.id == i) {
            return Err(format!(
                "Plugin `{plug_id}` does not provide variable `{i}`"
            ));
        }

        plugin.get_variable(i)
    }

    pub fn render_variable<S>(&self, id: S) -> String
    where
        S: AsRef<str>,
    {
        match self.try_resolve_variable(id) {
            Err(s) | Ok(s) => s,
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn try_run_action(&self, act: &RawDeckButtonAction) -> Result<(), ActionError> {
        let (plug_id, act_id) = act
            .id
            .split_once('.')
            .ok_or_else(|| ActionError::InvalidFormat(act.id.clone()))?;

        if plug_id.is_empty() || act_id.is_empty() {
            return Err(ActionError::InvalidFormat(act.id.clone()));
        }

        {
            self.plugins
                .read()
                .get(plug_id)
                .ok_or_else(|| ActionError::PluginNotFound(plug_id.into()))?
                .read()
                .run_action(act_id.to_string(), &act.args)?;
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
                r#type: var.r#type.to_string(),
            })
            .collect()
    }

    /// Get all variables of all plugins in the same vector
    pub fn get_all_variables_ungrouped(&self) -> Vec<PluginVariablesUngroupedData> {
        self.plugins
            .read()
            .values()
            .flat_map(|p| Self::get_variables_of(&p.read()))
            .collect()
    }

    /// Get all variables grouped by plugin with plugin `id` and `name`
    ///
    /// Does not include plugins without variables
    pub fn get_all_variables_grouped(&self) -> Vec<PluginVariablesGroupedData> {
        self.plugins
            .read()
            .values()
            .filter_map(|p| {
                let lock = p.read();
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
                        id: format!("{}.{}.{}", plugin.id, act.id, a.id),
                        name: a.name,
                        description: a.description,
                        r#type: a.r#type.to_string(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// Get all actions of all plugins in the same vector
    pub fn get_all_actions_ungrouped(&self) -> Vec<PluginActionsUngroupedData> {
        self.plugins
            .read()
            .values()
            .flat_map(|p| Self::get_actions_of(&p.read()))
            .collect()
    }

    /// Get all actions grouped by plugin with plugin `id` and `name`
    ///
    /// Does not include plugins without actions
    pub fn get_all_actions_grouped(&self) -> Vec<PluginActionsGroupedData> {
        self.plugins
            .read()
            .values()
            .filter_map(|p| {
                let lock = p.read();
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
            .read()
            .values()
            .map(|p| {
                let lock = p.read();
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

    #[allow(clippy::needless_pass_by_value)]
    pub fn get_enum_arg_variants(&self, id: String) -> Result<Vec<String>, String> {
        #[allow(clippy::expect_fun_call)]
        let (plug_id, act_id) = id
            .split_once('.')
            .expect(format!("Invalid arg id format: {id}").as_str());

        self.plugins
            .read()
            .get(plug_id)
            .expect("Failed to find plugin")
            .read()
            .get_enum_arg(act_id)
    }

    pub fn get_plugins_config(&self) -> HashMap<String, HashMap<String, String>> {
        self.plugins
            .read()
            .values()
            .map(|p| {
                let lock = p.read();
                todo!()
            })
            .collect()
    }
}
