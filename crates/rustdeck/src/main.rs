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

use crate::config::{load_config, paths, save_config};
use crate::deck::Deck;
use crate::server::http;

fn init_dirs() {
    tracing::info!("Plugins dir: {}", &*paths::PLUGINS);
    fs::create_dir_all(&*paths::PLUGINS).unwrap();
    fs::create_dir_all(&*paths::ICONS).unwrap();
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
