use futures::executor::block_on;
use media_session::{MediaInfo, MediaSession, traits::MediaSessionControls};
use rustdeck_common::{Plugin, actions, decl_action, decl_plugin, decl_variable, variables};

use std::panic::catch_unwind;

struct PluginState {
    player: MediaSession,
}

fn init() -> PluginState {
    PluginState {
        player: block_on(MediaSession::new()),
    }
}

fn update(_: &mut PluginState) {}

fn run_action(state: &PluginState, id: &str) {
    match id {
        "play_pause" => block_on(async { state.player.toggle_pause().await.unwrap() }),
        "play" => block_on(async { state.player.play().await.unwrap() }),
        "pause" => block_on(async { state.player.pause().await.unwrap() }),
        "stop" => block_on(async { state.player.stop().await.unwrap() }),
        "next" => block_on(async { state.player.next().await.unwrap() }),
        "previous" => block_on(async { state.player.prev().await.unwrap() }),
        _ => {}
    }
}

fn get_info() -> Option<MediaInfo> {
    let session_future = catch_unwind(async || MediaSession::new().await);
    if session_future.is_err() {
        return None;
    }
    let session = block_on(session_future.unwrap());
    Some(session.get_info())
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

#[unsafe(no_mangle)]
unsafe extern "C" fn build() -> *const Plugin {
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
