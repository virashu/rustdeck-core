#![feature(try_blocks)]
mod buttons;
mod deck;
mod error;
mod plugins;
mod server;
mod config;

use std::{sync::Arc, thread};

use deck::Deck;
use server::http_axum;

fn init_dirs() {
    std::fs::create_dir_all("./plugins").unwrap();
    std::fs::create_dir_all("./icons").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    init_dirs();

    let deck = Arc::new(Deck::new()?);

    let deck_ref = deck.clone();
    let deck_thread = thread::spawn(move || deck_ref.run());

    http_axum::build_and_run_thread(&deck, "0.0.0.0", 8989);

    deck_thread.join().unwrap();

    Ok(())
}
