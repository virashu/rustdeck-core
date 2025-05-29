use crate::plugins::PluginStore;

#[derive(Default)]
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

#[derive(Default)]
pub enum ButtonAction {
    #[default]
    None,
    DeckAction(String),
    PluginAction(String),
}

#[derive(Default)]
pub struct DeckButton {
    pub style: DeckButtonStyle,
    pub icon: Option<String>,
    pub content: String,
    pub on_click_action: ButtonAction,
}

static BUTTON_VAR_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\{(?<v>[a-zA-Z0-9_]+\.[a-zA-Z0-9_]+)\}").unwrap()
});

impl DeckButton {
    pub fn render_content(&self, plugins: &PluginStore) -> String {
        let input = &self.content;

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
            r#"{{"position": {{"y": {}, "x": {}}}, "style": {}, "content": "{}", "icon_image": {}}}"#,
            pos.0,
            pos.1,
            self.style.serialize(),
            self.render_content(plugins),
            self.icon
                .as_ref()
                .map_or("null".into(), |s| format!("\"{s}\""))
        )
    }
}
