use serde::Deserialize;

use crate::plugins::PluginStore;

static BUTTON_VAR_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\{(?<v>[a-zA-Z0-9_]+\.[a-zA-Z0-9_]+)\}").unwrap()
});

#[derive(Clone, Debug, Default, serde::Serialize)]
pub enum DeckButtonStyleTextAlign {
    #[default]
    Center,
    Left,
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

#[derive(Clone, Debug, serde::Serialize)]
pub struct DeckButtonStyle {
    pub text_align: DeckButtonStyleTextAlign,
    pub text_size: u32,
}

impl DeckButtonStyle {
    pub fn serialize(&self) -> String {
        format!(
            r#"{{"text_size": {}, "text_align": "{}"}}"#,
            self.text_size, self.text_align
        )
    }
}

impl Default for DeckButtonStyle {
    fn default() -> Self {
        Self {
            text_size: 24,
            text_align: DeckButtonStyleTextAlign::default(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RenderedDeckButton {
    pub position: DeckButtonPos,
    pub style: DeckButtonStyle,
    pub icon: Option<String>,
    pub content: String,
    pub on_click_action: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct DeckButton {
    pub style: DeckButtonStyle,
    pub icon: Option<String>,
    pub template: String,
    pub on_click_action: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
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

    pub fn serialize(&self, pos: (u32, u32), plugins: &PluginStore) -> String {
        format!(
            r#"{{"position": {{"y": {}, "x": {}}}, "style": {}, "content": "{}", "icon": {}, "action": {}}}"#,
            pos.0,
            pos.1,
            self.style.serialize(),
            self.render_content(plugins),
            self.icon
                .as_ref()
                .map_or("null".into(), |s| format!("\"{s}\"")),
            self.on_click_action
                .as_ref()
                .map_or_else(|| "null".into(), |s| format!("\"{s}\""))
        )
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
