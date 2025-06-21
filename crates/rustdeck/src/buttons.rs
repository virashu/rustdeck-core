use std::collections::HashMap;

use regex::Captures;
use serde::{Deserialize, Serialize};

use crate::plugins::PluginStore;

static BUTTON_VAR_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\{(?<var_id>[a-zA-Z0-9_]+\.[a-zA-Z0-9_]+)\}").unwrap()
});

/// Button content vertical align
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum DeckButtonStyleTextAlign {
    #[default]
    #[serde(alias = "center", rename = "center")]
    Center,
    #[serde(alias = "top", rename = "top")]
    Top,
    #[serde(alias = "bottom", rename = "bottom")]
    Bottom,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeckButtonStyle {
    pub text_align: DeckButtonStyleTextAlign,
    pub text_size: u32,
}

impl Default for DeckButtonStyle {
    fn default() -> Self {
        Self {
            text_size: 24,
            text_align: DeckButtonStyleTextAlign::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeckButtonPos {
    pub y: u32,
    pub x: u32,
}

impl DeckButtonPos {
    pub const fn from_yx(value: (u32, u32)) -> Self {
        Self {
            y: value.0,
            x: value.1,
        }
    }

    pub const fn as_yx(&self) -> (u32, u32) {
        (self.y, self.x)
    }
}

/// A deck button with its content rendered (interpolated with variables)
/// and position
#[derive(Clone, Debug, Serialize)]
pub struct RenderedDeckButton {
    pub position: DeckButtonPos,
    pub content: String,
    pub style: DeckButtonStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

pub struct VariableRenderer<'a> {
    cache: HashMap<String, String>,
    plugin_store: &'a PluginStore,
}

impl<'a> VariableRenderer<'a> {
    pub fn new(plugin_store: &'a PluginStore) -> Self {
        Self {
            cache: HashMap::new(),
            plugin_store,
        }
    }

    pub fn get(&mut self, id: &str) -> String {
        let var_opt = self.cache.get(id);

        if let Some(var) = var_opt {
            return var.clone();
        }

        let var = self.plugin_store.render_variable(id);
        self.cache.insert(id.to_string(), var.clone());
        var
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawDeckButtonAction {
    pub id: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RawDeckButton {
    pub template: String,
    pub style: DeckButtonStyle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_click_action: Option<RawDeckButtonAction>,
}

impl RawDeckButton {
    fn render_content(&self, vars: &mut VariableRenderer) -> String {
        if !self.template.contains('{') {
            return self.template.clone();
        }

        BUTTON_VAR_REGEX
            .replace_all(&self.template, |caps: &Captures| {
                let ident = &caps["var_id"];
                vars.get(ident)
            })
            .to_string()
    }

    pub fn render(&self, pos: (u32, u32), vars: &mut VariableRenderer) -> RenderedDeckButton {
        RenderedDeckButton {
            position: DeckButtonPos::from_yx(pos),
            style: self.style.clone(),
            icon: self.icon.clone(),
            content: self.render_content(vars),
        }
    }
}

#[derive(Deserialize)]
pub struct DeckButtonUpdate {
    pub template: String,
    pub style: DeckButtonStyle,
    pub icon: Option<String>,
    pub on_click_action: Option<RawDeckButtonAction>,
}
