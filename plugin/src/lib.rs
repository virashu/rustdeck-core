use std::{
    ffi::{c_char, c_void},
    mem::ManuallyDrop,
};

use common::{define_plugin, CPlugin};
// use common::RsPluginTrait as TPlugin;

// use std::ffi::{c_char, CString};

// #[repr(C)]
// pub struct Plugin {
//     name: String,
//     description: String,
//     id: String,
// }

// impl Plugin {
//     fn new() -> Self {
//         Self {
//             name: "Plugin".into(),
//             description: "Some plugin".into(),
//             id: "plugin_test".into(),
//         }
//     }
// }

// impl TPlugin for Plugin {
//     fn get_name(&self) -> *mut c_char {
//         let c_str = CString::new(self.name.clone()).unwrap();
//         Box::new(c_str).into_raw()
//     }
//     fn get_description(&self) -> *mut c_char {
//         let c_str = CString::new(self.description.clone()).unwrap();
//         Box::new(c_str).into_raw()
//     }
//     fn get_id(&self) -> *mut c_char {
//         let c_str = CString::new(self.id.clone()).unwrap();
//         Box::new(c_str).into_raw()
//     }
//     fn get_variables(&self) -> *mut c_char {
//         todo!()
//     }
//     fn get_actions(&self) -> *mut c_char {
//         todo!()
//     }
//     #[allow(unused_variables, reason = "WIP")]
//     fn execute_action(&self, id: *mut c_char) {
//         todo!()
//     }
//     fn update(&mut self) {
//         todo!()
//     }
// }

// #[no_mangle]
// pub extern "C" fn make() -> *mut dyn TPlugin {
//     Box::into_raw(Box::new(Plugin::new()))
// }

define_plugin! {
    name: "Plugin",
    description: "A sample plugin.",
    id: "plugin_test",
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
