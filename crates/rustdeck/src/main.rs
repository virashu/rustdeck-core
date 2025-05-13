mod deck;
mod error;
mod plugins;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let plugins = plugins::load_plugins_at(std::path::Path::new("./plugins")).unwrap();

    for (i, plugin) in plugins.iter().enumerate() {
        println!("{}) {}", i + 1, plugin.name);
    }

    println!("OK");

    Ok(())
}
