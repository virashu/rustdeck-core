use media_session::{MediaSession, traits::MediaSessionControls};
use rustdeck_common::{
    proto::Arg,
    {actions, decl_action, decl_plugin, decl_variable, export_plugin, variables},
};

struct PluginState {
    player: MediaSession,
}

fn init() -> PluginState {
    PluginState {
        player: MediaSession::new(),
    }
}

fn update(_: &mut PluginState) {}

fn run_action(state: &PluginState, id: &str, _: *const Arg) {
    _ = match id {
        "play_pause" => state.player.toggle_pause(),
        "play" => state.player.play(),
        "pause" => state.player.pause(),
        "stop" => state.player.stop(),
        "next" => state.player.next(),
        "previous" => state.player.prev(),
        _ => unreachable!(),
    };
}

fn get_variable(_: &PluginState, id: &str) -> String {
    let Ok(session) = std::panic::catch_unwind(MediaSession::new) else {
        println!("Caught a panic in rustdeck-media while trying to create a session");
        return String::new();
    };
    let media_info = session.get_info();

    // let media_info = state.player.get_info();

    match id {
        "title" => media_info.title,
        "artist" => media_info.artist,
        "state" => media_info.state,
        _ => unreachable!(),
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
