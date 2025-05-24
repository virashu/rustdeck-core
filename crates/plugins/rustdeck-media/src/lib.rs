#![allow(
    clippy::as_ptr_cast_mut,
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr,
    clippy::ptr_cast_constness,
    clippy::ref_as_ptr
)]

use std::{
    ffi::{c_char, c_void, CStr, CString},
    mem::ManuallyDrop,
    ptr::null_mut,
};

use futures::executor::block_on;

use media_session::{traits::MediaSessionControls, MediaSession};
use rustdeck_common::{define_plugin, CPlugin};

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

    (*value).as_ptr() as *mut c_char
}

struct PluginState {
    player: MediaSession,
}

unsafe extern "C" fn init() -> *mut c_void {
    let mut state = ManuallyDrop::new(Box::new(PluginState {
        player: block_on(MediaSession::new()),
    }));

    &mut (**state) as *mut PluginState as _
}

unsafe extern "C" fn update(state: *mut c_void) {
    let _state = &mut *(state as *mut PluginState);
}

unsafe extern "C" fn run_action(state: *mut c_void, id: *const c_char) {
    let state = &mut *(state as *mut PluginState);
    let id = CStr::from_ptr(id).to_str().unwrap();

    if id == "play_pause" {
        block_on(async { state.player.toggle_pause().await.unwrap() });
    }
}

unsafe extern "C" fn get_variable(state: *mut c_void, id: *const c_char) -> *mut c_char {
    let _state = &mut *(state as *mut PluginState);
    let id = CStr::from_ptr(id).to_str().unwrap();

    let media_info = block_on(MediaSession::new()).get_info();

    match id {
        "title" => string_to_ptr(media_info.title),
        "artist" => string_to_ptr(media_info.artist),
        "state" => string_to_ptr(media_info.state),
        _ => null_mut(),
    }
}
