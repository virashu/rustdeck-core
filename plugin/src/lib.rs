use std::{
    ffi::{c_char, c_void},
    mem::ManuallyDrop,
};

use common::{define_plugin, CPlugin};

define_plugin! {
    name: "Plugin",
    description: "A sample plugin.",
    id: "plugin_test",
    actions: [],
    variables: [],
    data: CPlugin {
        new,
        execute_action,
        update
    }
}

struct PluginState {}

unsafe extern "C" fn new() -> *mut c_void {
    let mut state = ManuallyDrop::new(Box::new(PluginState {}));

    &mut (**state) as *mut PluginState as _
}

unsafe extern "C" fn execute_action(state: *mut c_void, id: *const c_char) {}

unsafe extern "C" fn update(state: *mut c_void) {}
