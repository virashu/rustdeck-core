use std::sync::Arc;

use crate::deck::Deck;

pub struct DeckHttpServer {
    deck: Arc<Deck>,
    host: String,
    port: u32,
}

impl DeckHttpServer {
    pub fn new<S>(deck: Arc<Deck>, host: S, port: u32) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            deck,
            host: host.as_ref().to_string(),
            port,
        }
    }

    pub fn run(&self) {
        let mut app = saaba::App::new();

        //
        // Client
        //
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

            match deck_ref.handle_click_at(y, x) {
                Ok(()) => saaba::Response::from(""),
                Err(_) => saaba::Response::from_status(400u32),
            }
            .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.get_var("/api/client/icon/{icon}", move |_, params| {
            let icon_id = params.get("icon").unwrap();
            let icon_path = deck_ref.get_icon(icon_id);

            icon_path
                .map_or_else(saaba::Response::not_found, |path| {
                    saaba::Response::file(path).with_header("Content-Type", "image/png")
                })
                .with_header("Access-Control-Allow-Origin", "*")
        });

        //
        // Config
        //
        let deck_ref = self.deck.clone();
        app.get("/api/config/list/actions", move |_| {
            saaba::Response::from(deck_ref.serialize_actions())
                .with_header("Access-Control-Allow-Origin", "*")
        });

        let deck_ref = self.deck.clone();
        app.get("/api/config/list/variables", move |_| {
            saaba::Response::from(deck_ref.serialize_variables())
                .with_header("Access-Control-Allow-Origin", "*")
        });

        app.run(&self.host, self.port).unwrap();
    }
}
