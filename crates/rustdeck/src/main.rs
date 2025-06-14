#![feature(try_blocks)]
mod buttons;
mod config;
mod constants;
mod deck;
mod models;
mod plugins;
mod server;

use std::fs;
use std::path::Path;
use std::{sync::Arc, thread};

use crate::config::DeckConfig;
use crate::constants::{CONFIG_PATH, ICONS_DIR, PLUGIN_DIR};
use crate::deck::Deck;
use crate::server::http;

fn init_dirs() {
    fs::create_dir_all(PLUGIN_DIR).unwrap();
    fs::create_dir_all(ICONS_DIR).unwrap();
}

fn load_config() -> DeckConfig {
    if !Path::new(CONFIG_PATH).exists() {
        let config = DeckConfig::default();
        let config_ser = serde_json::to_string(&config).unwrap();
        fs::write(CONFIG_PATH, config_ser).expect("Failed to write default config");

        return config;
    }

    let config_ser = fs::read(CONFIG_PATH).expect("Failed to read config");
    serde_json::from_slice(&config_ser).expect("Failed to deserialize config")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    init_dirs();
    let config = load_config();

    let deck = Arc::new(Deck::new(config)?);

    let deck_ref = deck.clone();
    let deck_thread = thread::spawn(move || deck_ref.run());

    http::build_and_run_thread(&deck, "0.0.0.0", 8989);

    deck_thread.join().unwrap();

    Ok(())
}
