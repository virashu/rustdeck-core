mod config;
mod server;

use std::{fs, sync::Arc, thread, time::Duration};

use rustdeck::Deck;

use crate::config::{load_config, paths, save_config};

fn init_dirs() {
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

    let deck = Arc::new(Deck::new(
        config,
        save_config,
        &*paths::PLUGINS,
        &*paths::ICONS,
    )?);

    deck.init();

    let deck_ref = deck.clone();
    let deck_thread = thread::spawn(move || deck_ref.run(Duration::from_secs(1)));

    crate::server::http::build_and_run_thread(&deck, "0.0.0.0", 8989);

    deck_thread.join().unwrap();

    Ok(())
}
