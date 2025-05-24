use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crate::buttons::DeckButton;
use crate::plugins::PluginStore;

mod config {
    use std::time::Duration;

    pub const PLUGIN_DIR: &str = "./plugins";

    /// Update thread loop interval in millis
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
}

mod mock {
    use std::collections::HashMap;

    use crate::buttons::{DeckButton, DeckButtonStyle, DeckButtonStyleTextAlign};

    use super::DeckConfig;

    pub fn mock_config() -> DeckConfig {
        DeckConfig {
            http_host: "0.0.0.0".into(),
            http_port: 8989,
            cols: 5,
            rows: 3,
        }
    }

    pub fn mock_buttons() -> HashMap<(u32, u32), DeckButton> {
        HashMap::from([
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
            ])
    }
}

struct DeckServer {
    deck: Arc<Deck>,
    host: String,
    port: u32,
}

impl DeckServer {
    pub const fn new(deck: Arc<Deck>, host: String, port: u32) -> Self {
        Self { deck, host, port }
    }

    pub fn run(&self) {
        let mut app = saaba::App::new();

        let deck_ref = self.deck.clone();
        app.get("/api/client/config", move |_| {
            saaba::Response::from(deck_ref.serialize_config())
                .with_header("Content-Type", "application/json")
                .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.get("/api/client/buttons", move |_| {
            saaba::Response::from(deck_ref.serialize_buttons())
                .with_header("Content-Type", "application/json")
                .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.post_var("/api/client/click/{y}/{x}", move |_, params| {
            let y: u32 = params.get("y").unwrap().parse().unwrap();
            let x: u32 = params.get("x").unwrap().parse().unwrap();

            match deck_ref.handle_click(y, x) {
                Ok(()) => saaba::Response::from(""),
                Err(_) => saaba::Response::from_status(400u32),
            }
            .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.get_var("/api/client/icon/{icon}", move |_, params| {
            let icon_id = params.get("icon").unwrap();
            let icon_path = deck_ref.icons.get(icon_id.to_owned());

            icon_path
                .map_or_else(saaba::Response::not_found, |path| {
                    saaba::Response::file(path).with_header("Content-Type", "image/png")
                })
                .with_header("Access-Control-Allow-Origin", "*")
        });

        app.run(&self.host, self.port).unwrap();
    }
}

struct DeckConfig {
    pub http_host: String,
    pub http_port: u32,
    pub cols: u32,
    pub rows: u32,
}

pub struct Deck {
    config: DeckConfig,
    buttons: HashMap<(u32, u32), DeckButton>,
    plugin_store: PluginStore,
    icons: HashMap<String, String>,
}

impl Deck {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_store = PluginStore::new(config::PLUGIN_DIR)?;

        Ok(Self {
            buttons: mock::mock_buttons(),
            plugin_store,
            config: mock::mock_config(),
            icons: HashMap::from([("test_icon".into(), "icons/test_icon.png".into())]),
        })
    }

    fn server_thread(self: Arc<Self>) {
        let host = self.config.http_host.clone();
        let port = self.config.http_port;
        let server = DeckServer::new(self, host, port);
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
                self_.plugin_store.update_all();

                inst = Instant::now();
            }
        }
    }

    fn serialize_config(&self) -> String {
        format!(
            r#"{{"cols": {}, "rows": {}}}"#,
            self.config.cols, self.config.rows
        )
    }

    fn serialize_buttons(&self) -> String {
        let buttons: Vec<String> = self
            .buttons
            .iter()
            .map(|(k, b)| b.serialize(k.to_owned(), &self.plugin_store))
            .collect();

        format!("[{}]", buttons.join(", "))
    }

    fn handle_click(&self, y: u32, x: u32) -> Result<(), String> {
        let action_id = &self
            .buttons
            .get(&(y, x))
            .expect("Failed to get button at required index")
            .on_click_action;

        self.plugin_store.try_run_action(action_id)
    }
}
