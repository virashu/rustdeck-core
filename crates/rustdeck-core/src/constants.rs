use std::time::Duration;

/// ID of deck builtin actions
pub const DECK_ACTION_ID: &str = "deck";
/// ID with a dot at the end to avoid `format!`
pub const DECK_ACTION_PREFIX: &str = "deck.";
pub const DECK_ACTION_NAME: &str = "Deck";

/// Extension of a rustdeck plugin file
pub const DECK_PLUGIN_EXT: &str = ".deckplugin";

pub const PLUGIN_INIT_TIMEOUT: Duration = Duration::from_secs(10);
pub const PLUGIN_UPDATE_TIMEOUT: Duration = Duration::from_millis(100);
pub const PLUGIN_GET_VARIABLE_TIMEOUT: Duration = Duration::from_millis(100);
