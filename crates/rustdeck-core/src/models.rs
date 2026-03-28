use serde::Serialize;

use crate::{buttons::RenderedDeckButton, plugins::PluginDataType};

#[derive(Serialize, Clone)]
pub struct PluginActionArgsData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub r#type: PluginDataType,
}

#[derive(Serialize, Clone)]
pub struct PluginAction {
    /// Action id
    pub id: String,
    /// Action display name
    pub name: String,
    /// Action description
    pub description: String,
    /// Action arguments
    pub args: Vec<PluginActionArgsData>,
}

#[derive(Serialize, Clone)]
pub struct PluginActionGroup {
    /// Plugin id
    pub id: String,
    /// Plugin display name
    pub name: String,
    /// Actions of plugin
    pub actions: Vec<PluginAction>,
}

#[derive(Serialize, Clone)]
pub struct PluginVariable {
    /// Variable ID
    pub id: String,
    /// Variable description
    pub description: String,
    /// Variable type
    pub r#type: PluginDataType,
}

#[derive(Serialize, Clone)]
pub struct PluginVariableGroup {
    /// Plugin id
    pub id: String,
    /// Plugin display name
    pub name: String,
    /// Plugin variables
    pub variables: Vec<PluginVariable>,
}

#[derive(Serialize, Clone)]
pub struct PluginData {
    /// Plugin id
    pub id: String,
    /// Plugin display name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin variables
    pub variables: Vec<PluginVariable>,
    /// Actions of plugin
    pub actions: Vec<PluginAction>,
}

#[derive(Serialize, Clone)]
pub struct PluginConfigOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub r#type: PluginDataType,
}

#[derive(Serialize, Clone)]
pub struct PluginConfigOptionGroup {
    pub id: String,
    pub name: String,
    pub config_options: Vec<PluginConfigOption>,
}

#[derive(Debug, serde::Serialize)]
pub struct RenderedDeckScreen {
    pub screen: String,
    pub buttons: Vec<RenderedDeckButton>,
}
