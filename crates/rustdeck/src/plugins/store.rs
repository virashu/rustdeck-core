use std::{
    collections::HashMap,
    io,
    path::Path,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use parking_lot::RwLock;

use crate::{
    buttons::RawDeckButtonAction,
    models::{
        PluginAction, PluginActionArgsData, PluginActionGroup, PluginConfigOption,
        PluginConfigOptionGroup, PluginData, PluginVariable, PluginVariableGroup,
    },
    plugins::error::ActionError,
};

use super::{Plugin, load_plugins_at};

pub struct PluginStore {
    plugins: RwLock<HashMap<String, Arc<RwLock<Plugin>>>>,
    plugins_uninit: RwLock<Vec<Arc<RwLock<Plugin>>>>,
}

fn timeout<T>(handle: JoinHandle<T>, dur: Duration) -> Result<T, ()> {
    let timer = std::time::Instant::now();

    while !handle.is_finished() && timer.elapsed() < dur {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if handle.is_finished() {
        Ok(handle.join().unwrap())
    } else {
        Err(())
    }
}

impl PluginStore {
    /// Create a new plugin store and load all the plugins.
    ///
    /// # Errors
    /// Error is returned if path cannot be read.
    pub fn new<S>(path: S) -> Result<Self, io::Error>
    where
        S: AsRef<str>,
    {
        tracing::info!("Loading plugins...");

        let plugins = load_plugins_at(Path::new(path.as_ref()))?;
        let plugins = RwLock::new(
            plugins
                .into_iter()
                .map(|p| (p.id.clone(), Arc::new(RwLock::new(p))))
                .collect(),
        );

        Ok(Self {
            plugins,
            plugins_uninit: RwLock::new(Vec::new()),
        })
    }

    pub fn init_all(&self) {
        tracing::info!("Initializing plugins...");

        let mut uninit = Vec::new();

        {
            self.plugins.read().iter().for_each(|(id, p)| {
                tracing::info!("Initializing plugin {id:?}...");

                let ref_ = p.clone();
                let handle = thread::spawn(move || ref_.write().init());

                match timeout(handle, Duration::from_secs(10)) {
                    // Ok
                    Ok(Ok(())) => {
                        tracing::info!("Initialized plugin {id:?}");
                    }
                    // Error in plugin
                    Ok(Err(e)) => {
                        tracing::warn!("Failed to initialize plugin {id:?}: {e}");
                        uninit.push(id.clone());
                    }
                    // Timeout
                    Err(()) => {
                        tracing::warn!("Plugin {id:?} took to long to initialize.");
                    }
                }
            });
        }

        tracing::info!("Initialized plugins");

        {
            let mut plugins = self.plugins.write();
            let mut uninit_vec = self.plugins_uninit.write();

            for id in uninit {
                #[allow(clippy::missing_panics_doc)]
                let plugin = plugins.remove(&id).unwrap();
                uninit_vec.push(plugin);
            }
        }
    }

    pub fn update_all(&self) {
        self.plugins.read().values().for_each(|p| {
            let mut lock = p.write();
            _ = lock.update().inspect_err(|e| {
                tracing::warn!(
                    "A error occurred while updating plugin {:?}: {}",
                    lock.id,
                    e
                );
            });
        });
    }

    pub fn try_resolve_variable<S>(&self, id: S) -> Result<String, String>
    where
        S: AsRef<str>,
    {
        let (plug_id, i) = id.as_ref().split_once('.').ok_or("Wrong variable format")?;
        let plugins = self.plugins.read();
        let plugin = plugins
            .get(plug_id)
            .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?;

        if !plugin.read().variables.iter().any(|v| v.id == i) {
            return Err(format!(
                "Plugin `{plug_id}` does not provide variable `{i}`"
            ));
        }

        let ref_ = plugin.clone();
        let id_ = i.to_string();
        let handle = thread::spawn(move || ref_.read().get_variable(id_));

        match timeout(handle, Duration::from_secs(10)) {
            Ok(Ok(s)) => Ok(s),
            Ok(Err(e)) => Err(e),
            Err(()) => Err(String::from("Timeout")),
        }
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
    fn get_variables_of(plugin: &Plugin) -> Vec<PluginVariable> {
        plugin
            .variables
            .iter()
            .map(|var| PluginVariable {
                id: format!("{}.{}", plugin.id, var.id),
                description: var.description.clone(),
                r#type: var.r#type.clone(),
            })
            .collect()
    }

    /// Get all variables of all plugins in the same vector
    pub fn get_all_variables_ungrouped(&self) -> Vec<PluginVariable> {
        self.plugins
            .read()
            .values()
            .flat_map(|p| Self::get_variables_of(&p.read()))
            .collect()
    }

    /// Get all variables grouped by plugin with plugin `id` and `name`
    ///
    /// Does not include plugins without variables
    pub fn get_all_variables_grouped(&self) -> Vec<PluginVariableGroup> {
        self.plugins
            .read()
            .values()
            .filter_map(|p| {
                let lock = p.read();
                let variables = Self::get_variables_of(&lock);
                if variables.is_empty() {
                    None
                } else {
                    Some(PluginVariableGroup {
                        id: lock.id.clone(),
                        name: lock.name.clone(),
                        variables,
                    })
                }
            })
            .collect()
    }

    /// Get actions of a plugin
    fn get_actions_of(plugin: &Plugin) -> Vec<PluginAction> {
        plugin
            .actions
            .iter()
            .map(|act| PluginAction {
                id: format!("{}.{}", plugin.id, act.id),
                name: act.name.clone(),
                description: act.description.clone(),
                args: act
                    .args
                    .iter()
                    .map(|a| PluginActionArgsData {
                        id: format!("{}.{}.{}", plugin.id, act.id, a.id),
                        name: a.name.clone(),
                        description: a.description.clone(),
                        r#type: a.r#type.clone(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// Get all actions of all plugins in the same vector
    pub fn get_all_actions_ungrouped(&self) -> Vec<PluginAction> {
        self.plugins
            .read()
            .values()
            .flat_map(|p| Self::get_actions_of(&p.read()))
            .collect()
    }

    /// Get all actions grouped by plugin with plugin `id` and `name`
    ///
    /// Does not include plugins without actions
    pub fn get_all_actions_grouped(&self) -> Vec<PluginActionGroup> {
        self.plugins
            .read()
            .values()
            .filter_map(|p| {
                let lock = p.read();
                let actions = Self::get_actions_of(&lock);
                if actions.is_empty() {
                    None
                } else {
                    Some(PluginActionGroup {
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
            .ok_or_else(|| format!("Invalid arg id format: {id}"))?;

        self.plugins
            .read()
            .get(plug_id)
            .ok_or_else(|| String::from("Failed to find plugin"))?
            .read()
            .get_enum_arg(act_id)
    }

    fn get_config_options_of(plugin: &Plugin) -> Vec<PluginConfigOption> {
        plugin
            .config_options
            .iter()
            .map(|opt| PluginConfigOption {
                id: opt.id.clone(),
                name: opt.name.clone(),
                description: opt.description.clone(),
                r#type: opt.r#type.clone(),
            })
            .collect()
    }

    pub fn get_all_config_options_ungrouped(&self) -> Vec<PluginConfigOption> {
        self.plugins
            .read()
            .values()
            .flat_map(|p| Self::get_config_options_of(&p.read()))
            .collect()
    }

    pub fn get_all_config_options_grouped(&self) -> Vec<PluginConfigOptionGroup> {
        self.plugins
            .read()
            .values()
            .filter_map(|p| {
                let lock = p.read();
                let options = Self::get_config_options_of(&lock);
                if options.is_empty() {
                    None
                } else {
                    Some(PluginConfigOptionGroup {
                        id: lock.id.clone(),
                        name: lock.name.clone(),
                        config_options: options,
                    })
                }
            })
            .collect()
    }

    /// Get full plugins config grouped by plugin id and without prefix
    pub fn get_plugins_config(&self) -> HashMap<String, HashMap<String, String>> {
        self.plugins
            .read()
            .values()
            .map(|p| {
                let mut lock = p.write();
                let opts_ids = lock
                    .config_options
                    .iter()
                    .map(|opt| opt.id.clone())
                    .collect::<Vec<String>>();
                let opts = opts_ids
                    .iter()
                    .map(|opt_id| (opt_id.clone(), lock.get_config_value(opt_id).unwrap()))
                    .collect();

                (lock.id.clone(), opts)
            })
            .collect()
    }

    pub fn set_config(&self, id: impl AsRef<str>, value: impl AsRef<str>) -> Result<(), String> {
        let (plug_id, i) = id
            .as_ref()
            .split_once('.')
            .ok_or_else(|| String::from("Wrong config format"))?;

        let plugins = self.plugins.read();
        let mut plugin = plugins
            .get(plug_id)
            .ok_or_else(|| format!("Cannot find plugin: `{plug_id}`"))?
            .write();

        if !plugin.variables.iter().any(|v| v.id == i) {
            return Err(format!(
                "Plugin `{plug_id}` does not have config option `{i}`"
            ));
        }

        plugin
            .set_config_value(i, value.as_ref().to_owned())
            .inspect_err(|e| tracing::warn!("Failed to set config: {}", e))
    }
}
