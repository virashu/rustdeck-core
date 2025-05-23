use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::plugins::{self, Plugin};

mod config {
    use std::time::Duration;

    pub const PLUGIN_DIR: &str = "./plugins";

    /// Update thread loop interval in millis
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
}

#[derive(Default)]
enum DeckButtonStyleTextAlign {
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

struct DeckButtonStyle {
    text_align: DeckButtonStyleTextAlign,
    text_size: u32,
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
struct DeckButton {
    style: DeckButtonStyle,
    content: String,
}

static BUTTON_VAR_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\{(?<v>[a-zA-Z0-9_]+\.[a-zA-Z0-9_]+)\}").unwrap()
});

impl DeckButton {
    pub fn render_content(&self, deck: &Arc<Deck>) -> String {
        let input = &self.content;

        let a: Vec<(String, String)> = BUTTON_VAR_REGEX
            .captures_iter(input)
            .map(|m| {
                let ident = &m["v"];
                let value = deck.render_variable(ident);
                (ident.to_owned(), value)
            })
            .collect();

        let mut output = String::from(input);

        for (s, var) in a {
            output = output.replace(&format!("{{{s}}}"), &var);
        }

        output
    }

    pub fn serialize(&self, pos: (u32, u32), deck: &Arc<Deck>) -> String {
        format!(
            r#"{{"position": {{"y": {}, "x": {}}}, "style": {}, "content": "{}"}}"#,
            pos.0,
            pos.1,
            self.style.serialize(),
            self.render_content(deck)
        )
    }
}

struct DeckServer {
    deck: Arc<Deck>,
}

impl DeckServer {
    pub const fn new(deck: Arc<Deck>) -> Self {
        Self { deck }
    }

    pub fn run(&self) {
        let mut app = saaba::App::new();

        let deck_ref = self.deck.clone();
        app.get("/", move |_| {
            saaba::Response::from(Deck::serialize_buttons(deck_ref.clone())).with_header("Content-Type", "application/json")
        });

        app.run("0.0.0.0", 3000).unwrap();
    }
}

pub struct Deck {
    buttons: HashMap<(u32, u32), DeckButton>,
    plugins: HashMap<String, Mutex<Plugin>>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let plugins = plugins::load_plugins_at(std::path::Path::new(config::PLUGIN_DIR))?;
        let plugins = plugins
            .into_iter()
            .map(|p| (p.id.clone(), Mutex::new(p)))
            .collect();

        Ok(Self {
            buttons: HashMap::from([(
                (0, 0),
                DeckButton {
                    style: DeckButtonStyle::default(),
                    content: String::from("Counter: {plugin_test.counter}"),
                },
            )]),
            plugins,
        })
    }

    fn server_thread(self: Arc<Self>) {
        let server = DeckServer::new(self);
        server.run();
    }

    pub fn run(self) {
        let self_ = Arc::new(self);

        let c = self_.clone();
        thread::spawn(move || {
            Self::server_thread(c);
        });

        let mut inst = Instant::now();

        loop {
            if inst.elapsed() > config::UPDATE_INTERVAL {
                self_
                    .plugins
                    .values()
                    .for_each(|p| p.lock().unwrap().update());

                self_.try_run_action("plugin_test.increment").unwrap();

                inst = Instant::now();
            }
        }
    }

    fn try_resolve_variable(&self, id: &str) -> Result<String, String> {
        let (plug_id, i) = id.split_once('.').ok_or("Wrong variable format")?;
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

        Ok(plugin.get_variable(i.to_string()))
    }

    fn render_variable(&self, id: &str) -> String {
        match self.try_resolve_variable(id) {
            Err(s) | Ok(s) => s,
        }
    }

    fn try_run_action(&self, id: &str) -> Result<(), String> {
        let (plug_id, i) = id.split_once('.').ok_or("Wrong action format")?;
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

    fn serialize_buttons(self: Arc<Self>) -> String {
        let buttons: Vec<String> = self
            .buttons
            .iter()
            .map(|(k, b)| b.serialize(k.to_owned(), &self.clone()))
            .collect();
        format!(r#"{{"buttons": [{}]}}"#, buttons.join(", "))
    }
}
