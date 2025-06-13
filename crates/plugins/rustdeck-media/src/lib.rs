use futures::executor::block_on;
use media_session::{MediaSession, traits::MediaSessionControls};
use rustdeck_common::{CPlugin, define_plugin};

use std::{
    ffi::{CStr, CString, c_char, c_void},
    mem::ManuallyDrop,
    panic::catch_unwind,
    ptr::null_mut,
};

define_plugin! {
    name: "Plugin",
    description: "A sample plugin.",
    id: "rustdeck_media",
    actions: "play_pause",
    variables: "title, artist, state",
    data: CPlugin {
        init,
        run_action,
        get_variable,
        update
    }
}

fn string_to_ptr(s: String) -> *mut c_char {
    let value = ManuallyDrop::new(Box::new(CString::new(s).unwrap()));

    (*value).as_ptr().cast_mut()
}

struct PluginState {
    player: MediaSession,
}

unsafe extern "C" fn init() -> *mut c_void {
    let mut state = ManuallyDrop::new(Box::new(PluginState {
        player: block_on(MediaSession::new()),
    }));

    (&raw mut (**state)).cast()
}

unsafe extern "C" fn update(state: *mut c_void) {
    let _state = unsafe { &mut *state.cast::<PluginState>() };
}

unsafe extern "C" fn run_action(state: *mut c_void, id: *const c_char) {
    let state = unsafe { &mut *state.cast::<PluginState>() };
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap() };

    if id == "play_pause" {
        block_on(async { state.player.toggle_pause().await.unwrap() });
    }
}

unsafe extern "C" fn get_variable(state: *mut c_void, id: *const c_char) -> *mut c_char {
    let _state = unsafe { &mut *state.cast::<PluginState>() };
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap() };

    let Ok(media_info) = catch_unwind(|| block_on(MediaSession::new()).get_info()) else {
        return null_mut();
    };

    match id {
        "title" => string_to_ptr(media_info.title),
        "artist" => string_to_ptr(media_info.artist),
        "state" => string_to_ptr(media_info.state),
        _ => null_mut(),
    }
}
