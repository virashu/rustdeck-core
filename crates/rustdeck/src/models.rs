use serde::Serialize;

#[derive(Serialize)]
pub struct PluginActionsData {
    /// Plugin id
    pub id: String,
    /// Plugin display name
    pub name: String,
    /// Actions of plugin
    pub actions: Vec<String>,
}
