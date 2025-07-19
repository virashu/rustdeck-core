#![allow(clippy::unnecessary_wraps)]
#![allow(unsafe_op_in_unsafe_fn)]

use media_session::{MediaSession, traits::MediaSessionControls};
use rustdeck_common::{
    Args, Type,
    builder::{Action, PluginBuilder, Variable},
    decorate_fn_get_variable, decorate_fn_init, decorate_fn_run_action, decorate_fn_update,
    export_plugin,
};

struct PluginState {
    player: MediaSession,
}

fn init() -> Result<PluginState, Box<dyn std::error::Error>> {
    Ok(PluginState {
        player: MediaSession::new(),
    })
}

fn update(state: &mut PluginState) {
    state.player.update();
}

fn run_action(state: &PluginState, id: &str, _: &Args) -> Result<(), Box<dyn std::error::Error>> {
    match id {
        "play_pause" => state.player.toggle_pause()?,
        "play" => state.player.play()?,
        "pause" => state.player.pause()?,
        "stop" => state.player.stop()?,
        "next" => state.player.next()?,
        "previous" => state.player.prev()?,
        _ => unreachable!(),
    }

    Ok(())
}

fn get_variable(state: &PluginState, id: &str) -> Result<String, String> {
    let media_info = state.player.get_info();

    Ok(match id {
        "title" => media_info.title,
        "artist" => media_info.artist,
        "state" => media_info.state,
        _ => unreachable!(),
    })
}

export_plugin! {
    PluginBuilder::new(
        "rustdeck_media",
        "Rustdeck Media",
        "A plugin for media management (music, video, etc.)",
    )
        .init(decorate_fn_init!(init))
        .update(decorate_fn_update!(update))
        .get_variable(decorate_fn_get_variable!(get_variable))
        .run_action(decorate_fn_run_action!(run_action))
        .variable(Variable::new("title", "Title", Type::String))
        .variable(Variable::new("artist", "Artist", Type::String))
        .variable(Variable::new("state", "State", Type::String))
        .action(Action::new(
            "play_pause",
            "Pause toggle",
            "Toggle play/pause",
        ))
        .action(Action::new("play", "Play", "Play media"))
        .action(Action::new("pause", "Pause", "Pause media"))
        .action(Action::new("stop", "Stop", "Stop playback"))
        .action(Action::new("previous", "Previous", "Previous track"))
        .action(Action::new("next", "Next", "Next track"))
        .build()
        .unwrap()
}
