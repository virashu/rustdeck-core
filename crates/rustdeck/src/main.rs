#![feature(try_blocks)]

use deck::Deck;

mod buttons;
mod deck;
mod error;
mod plugins;

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

    let deck = Deck::new()?;
    deck.run();

    Ok(())
}
