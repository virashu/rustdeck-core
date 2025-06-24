use media_session::{MediaInfo, MediaSession, traits::MediaSessionControls};
use rustdeck_common::{actions, decl_action, decl_plugin, decl_variable, export_plugin, variables};

use std::panic::catch_unwind;

struct PluginState {
    player: MediaSession,
}

fn init() -> PluginState {
    PluginState {
        player: MediaSession::new(),
    }
}

fn update(_: &mut PluginState) {}

fn run_action(state: &PluginState, id: &str) {
    match id {
        "play_pause" => state.player.toggle_pause().unwrap(),
        "play" => state.player.play().unwrap(),
        "pause" => state.player.pause().unwrap(),
        "stop" => state.player.stop().unwrap(),
        "next" => state.player.next().unwrap(),
        "previous" => state.player.prev().unwrap(),
        _ => {}
    }
}

fn get_info() -> Option<MediaInfo> {
    let session = catch_unwind(MediaSession::new);
    session.map_or(None, |session| Some(session.get_info()))
}

fn get_variable(_: &PluginState, id: &str) -> String {
    let Some(media_info) = get_info() else {
        return String::new();
    };

    match id {
        "title" => media_info.title,
        "artist" => media_info.artist,
        "state" => media_info.state,
        _ => String::new(),
    }
}

export_plugin! {
    decl_plugin! {
        id: "rustdeck_media",
        name: "RustDeck Media",
        desc: "A plugin for media management (music, video, etc.)",
        variables: variables!(
            decl_variable! {
                id: "title",
                desc: "Title",
                vtype: "string",
            },
            decl_variable! {
                id: "artist",
                desc: "Artist",
                vtype: "string",
            },
            decl_variable! {
                id: "state",
                desc: "State",
                vtype: "string",
            },
        ),
        actions: actions!(
            decl_action! {
                id: "play_pause",
                name: "Pause toggle",
                desc: "Toggle play/pause"
            },
            decl_action! {
                id: "play",
                name: "Play",
                desc: "Play media"
            },
            decl_action! {
                id: "pause",
                name: "Pause",
                desc: "Pause media"
            },
            decl_action! {
                id: "stop",
                name: "Stop",
                desc: "Stop playback"
            },
            decl_action! {
                id: "previous",
                name: "Previous",
                desc: "Previous track"
            },
            decl_action! {
                id: "next",
                name: "Next",
                desc: "Next track"
            },
        ),

        fn_init: init,
        fn_update: update,
        fn_get_variable: get_variable,
        fn_run_action: run_action,
    }
}
