#![feature(try_blocks)]
mod buttons;
mod config;
mod constants;
mod deck;
mod models;
mod plugins;
mod server;

use std::fs;
use std::{sync::Arc, thread};

use crate::config::{load_config, save_config};
use crate::constants::{ICONS_DIR, PLUGIN_DIR};
use crate::deck::Deck;
use crate::server::http;

fn init_dirs() {
    fs::create_dir_all(PLUGIN_DIR).unwrap();
    fs::create_dir_all(ICONS_DIR).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    init_dirs();
    let config = load_config();

    let deck = Arc::new(Deck::new(config, save_config)?);

    let deck_ref = deck.clone();
    let deck_thread = thread::spawn(move || deck_ref.run());

    http::build_and_run_thread(&deck, "0.0.0.0", 8989);

    deck_thread.join().unwrap();

    Ok(())
}
