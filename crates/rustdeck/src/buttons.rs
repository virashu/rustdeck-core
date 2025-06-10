use serde::{Deserialize, Serialize};

use crate::plugins::PluginStore;

static BUTTON_VAR_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\{(?<v>[a-zA-Z0-9_]+\.[a-zA-Z0-9_]+)\}").unwrap()
});

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum DeckButtonStyleTextAlign {
    #[default]
    #[serde(alias = "center", rename = "center")]
    Center,
    #[serde(alias = "left", rename = "left")]
    Left,
    #[serde(alias = "right", rename = "right")]
    Right,
}

impl std::fmt::Display for DeckButtonStyleTextAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Center => "center",
                Self::Left => "left",
                Self::Right => "right",
            }
        )
    }
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
    pub x: u32,
    pub y: u32,
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
    pub style: DeckButtonStyle,
    pub icon: Option<String>,
    pub content: String,
    pub on_click_action: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DeckButton {
    pub style: DeckButtonStyle,
    pub icon: Option<String>,
    pub template: String,
    pub on_click_action: Option<String>,
}

impl DeckButton {
    fn render_content(&self, plugins: &PluginStore) -> String {
        let input = &self.template;

        let a: Vec<(String, String)> = BUTTON_VAR_REGEX
            .captures_iter(input)
            .map(|m| {
                let ident = &m["v"];
                let value = plugins.render_variable(ident);
                (ident.to_owned(), value)
            })
            .collect();

        let mut output = String::from(input);

        for (s, var) in a {
            output = output.replace(&format!("{{{s}}}"), &var);
        }

        output
    }

    pub fn render(&self, pos: (u32, u32), plugins: &PluginStore) -> RenderedDeckButton {
        RenderedDeckButton {
            position: DeckButtonPos::from_yx(pos),
            style: self.style.clone(),
            icon: self.icon.clone(),
            content: self.render_content(plugins),
            on_click_action: self.on_click_action.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct DeckButtonUpdate {
    pub template: String,
    pub on_click_action: Option<String>,
}
