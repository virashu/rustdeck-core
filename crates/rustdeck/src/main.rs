use deck::Deck;

mod deck;
mod error;
mod plugins;

fn init_plugin_dir() {
    std::fs::create_dir_all("./plugins").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    init_plugin_dir();

    let deck = Deck::new()?;
    deck.run();

    Ok(())
}
