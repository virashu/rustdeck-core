use std::{
    ffi::{c_char, c_void, CStr, CString},
    mem::ManuallyDrop,
    ptr::null_mut,
};

use rustdeck_common::{define_plugin, CPlugin};

define_plugin! {
    name: "Plugin",
    description: "A sample plugin.",
    id: "plugin_test",
    actions: "increment, clear",
    variables: "counter",
    data: CPlugin {
        init,
        run_action,
        get_variable,
        update
    }
}

struct PluginState {
    counter: i32,
}

unsafe extern "C" fn init() -> *mut c_void {
    let mut state = ManuallyDrop::new(Box::new(PluginState { counter: 0 }));

    (&raw mut (**state)).cast()
}

unsafe extern "C" fn update(state: *mut c_void) {
    let _state = unsafe { &mut *state.cast::<PluginState>() };
}

unsafe extern "C" fn run_action(state: *mut c_void, id: *const c_char) {
    let state = unsafe { &mut *state.cast::<PluginState>() };
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap() };

    match id {
        "increment" => {
            state.counter += 1;
        }
        "clear" => {
            state.counter = 0;
        }
        _ => {}
    }
}

unsafe extern "C" fn get_variable(state: *mut c_void, id: *const c_char) -> *mut c_char {
    let state = unsafe { &mut *state.cast::<PluginState>() };
    let id = unsafe { CStr::from_ptr(id).to_str().unwrap() };

    if id == "counter" {
        let counter_value =
            ManuallyDrop::new(Box::new(CString::new(state.counter.to_string()).unwrap()));

        return (*counter_value).as_ptr().cast_mut();
    }

    null_mut()
}
