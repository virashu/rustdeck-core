use std::collections::HashMap;

use crate::buttons::{DeckButton, DeckButtonStyle, DeckButtonStyleTextAlign};
use crate::config::DeckDimensionConfig;

pub const fn mock_config() -> DeckDimensionConfig {
    DeckDimensionConfig { cols: 5, rows: 3 }
}

pub fn mock_buttons_screen_1() -> HashMap<(u32, u32), DeckButton> {
    HashMap::from([
        (
            (1, 1),
            DeckButton {
                style: DeckButtonStyle {
                    text_align: DeckButtonStyleTextAlign::Right,
                    ..Default::default()
                },
                template: String::from("Counter: {plugin_test.counter}"),
                on_click_action: Some(String::from("plugin_test.increment")),
                icon: None,
            },
        ),
        (
            (1, 2),
            DeckButton {
                style: DeckButtonStyle {
                    text_align: DeckButtonStyleTextAlign::Left,
                    ..Default::default()
                },
                template: String::from("Clear counter"),
                on_click_action: Some(String::from("plugin_test.clear")),
                icon: None,
            },
        ),
        (
            (1, 3),
            DeckButton {
                style: DeckButtonStyle::default(),
                template: String::new(),
                on_click_action: None,
                icon: Some("test_icon".into()),
            },
        ),
        (
            (1, 4),
            DeckButton {
                style: DeckButtonStyle::default(),
                template: String::from("Switch to screen 2"),
                on_click_action: Some(String::from("deck.switch_screen:screen_2")),
                icon: None,
            },
        ),
        (
            (2, 3),
            DeckButton {
                style: DeckButtonStyle::default(),
                template: String::from(
                    "State: {rustdeck_media.state}\\nTitle: '{rustdeck_media.title}'\\nArtist: '{rustdeck_media.artist}'",
                ),
                on_click_action: Some(String::from("rustdeck_media.play_pause")),
                icon: None,
            },
        ),
    ])
}

pub fn mock_buttons_screen_2() -> HashMap<(u32, u32), DeckButton> {
    HashMap::from([(
        (1, 1),
        DeckButton {
            style: DeckButtonStyle::default(),
            template: String::from("Switch to screen 1"),
            on_click_action: Some(String::from("deck.switch_screen:default")),
            icon: None,
        },
    )])
}
