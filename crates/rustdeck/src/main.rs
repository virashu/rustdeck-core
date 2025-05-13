use deck::Deck;

mod deck;
mod error;
mod plugins;

fn init_plugin_dir() {
    std::fs::create_dir_all("./plugins").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_plugin_dir();

    let mut deck = Deck::new()?;
    deck.run();

    // let plugins = plugins::load_plugins_at(std::path::Path::new("./plugins")).unwrap();

    // for (i, plugin) in plugins.iter().enumerate() {
    //     println!("{}) {}", i + 1, plugin.name);
    // }

    println!("OK");

    Ok(())
}
