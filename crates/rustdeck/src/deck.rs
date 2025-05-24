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
    icon: Option<String>,
    content: String,
    on_click_action: String,
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
            r#"{{"position": {{"y": {}, "x": {}}}, "style": {}, "content": "{}", "icon_image": {}}}"#,
            pos.0,
            pos.1,
            self.style.serialize(),
            self.render_content(deck),
            self.icon
                .as_ref()
                .map_or("null".into(), |s| format!("\"{s}\""))
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
        app.get("/api/config", move |_| {
            saaba::Response::from(Deck::serialize_config(&deck_ref))
                .with_header("Content-Type", "application/json")
                .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.get("/api/buttons", move |_| {
            saaba::Response::from(Deck::serialize_buttons(&deck_ref))
                .with_header("Content-Type", "application/json")
                .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.post_var("/api/click/{y}/{x}", move |_, params| {
            let y: u32 = params.get("y").unwrap().parse().unwrap();
            let x: u32 = params.get("x").unwrap().parse().unwrap();

            match Deck::handle_click(&deck_ref, y, x) {
                Ok(()) => saaba::Response::from(""),
                Err(_) => saaba::Response::from_status(400u32),
            }
            .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.get_var("/api/icon/{icon}", move |_, params| {
            let icon_id = params.get("icon").unwrap();
            let icon_path = deck_ref.icons.get(icon_id.to_owned());

            icon_path
                .map_or_else(saaba::Response::not_found, |path| {
                    saaba::Response::file(path).with_header("Content-Type", "image/png")
                })
                .with_header("Access-Control-Allow-Origin", "*")
        });

        app.run("0.0.0.0", 8989).unwrap();
    }
}

struct DeckConfig {
    pub cols: u32,
    pub rows: u32,
}

pub struct Deck {
    config: DeckConfig,
    buttons: HashMap<(u32, u32), DeckButton>,
    plugins: HashMap<String, Mutex<Plugin>>,
    icons: HashMap<String, String>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let plugins = plugins::load_plugins_at(std::path::Path::new(config::PLUGIN_DIR))?;
        let plugins = plugins
            .into_iter()
            .map(|p| (p.id.clone(), Mutex::new(p)))
            .collect();

        Ok(Self {
            buttons: HashMap::from([
                (
                    (1, 1),
                    DeckButton {
                        style: DeckButtonStyle {
                            text_align: DeckButtonStyleTextAlign::Right,
                            ..Default::default()
                        },
                        content: String::from("Counter: {plugin_test.counter}"),
                        on_click_action: String::from("plugin_test.increment"),
                        icon: None,
                    },
                ),
                (
                    (1, 2),
                    DeckButton {
                        style: DeckButtonStyle {
                            text_align: DeckButtonStyleTextAlign::Left,
                            ..Default::default()
                        },
                        content: String::from("Clear counter"),
                        on_click_action: String::from("plugin_test.clear"),
                        icon: None,
                    },
                ),
                (
                    (1, 3),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        content: String::new(),
                        on_click_action: String::new(),
                        icon: Some("test_icon".into()),
                    },
                ),
                (
                    (2, 3),
                    DeckButton {
                        style: DeckButtonStyle::default(),
                        content: String::from(
                            "State: {rustdeck_media.state}\\nTitle: '{rustdeck_media.title}'\\nArtist: '{rustdeck_media.artist}'",
                        ),
                        on_click_action: String::from("rustdeck_media.play_pause"),
                        icon: None,
                    },
                ),
            ]),
            plugins,
            config: DeckConfig { cols: 5, rows: 3 },
            icons: HashMap::from([("test_icon".into(), "icons/test_icon.png".into())])
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
        let (plug_id, i) = id
            .split_once('.')
            .ok_or_else(|| format!("Wrong action format: `{id}`"))?;

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

    fn serialize_config(self: &Arc<Self>) -> String {
        format!(
            r#"{{"cols": {}, "rows": {}}}"#,
            self.config.cols, self.config.rows
        )
    }

    fn serialize_buttons(self: &Arc<Self>) -> String {
        let buttons: Vec<String> = self
            .buttons
            .iter()
            .map(|(k, b)| b.serialize(k.to_owned(), &self.clone()))
            .collect();
        format!("[{}]", buttons.join(", "))
    }

    fn handle_click(self: &Arc<Self>, y: u32, x: u32) -> Result<(), String> {
        let action_id = &self
            .buttons
            .get(&(y, x))
            .expect("Failed to get button at required index")
            .on_click_action;

        self.try_run_action(action_id)
    }
}
