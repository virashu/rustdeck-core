use std::ffi::{c_char, c_void};

pub mod interface {
    pub const NAME_IDENT: &[u8] = b"get_name";
    pub const DESCRIPTION_IDENT: &[u8] = b"get_description";
    pub const ID_IDENT: &[u8] = b"get_id";
    pub const PLUGIN_IDENT: &[u8] = b"PLUGIN";

    pub const ACTIONS: &[u8] = b"get_actions";
    pub const VARIABLES: &[u8] = b"get_variables";
}

#[repr(C)]
pub struct CPlugin {
    pub init: unsafe extern "C" fn() -> *mut c_void,

    pub update: unsafe extern "C" fn(state: *mut c_void),

    pub run_action: unsafe extern "C" fn(state: *mut c_void, id: *const c_char),
    pub get_variable: unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> *mut c_char,
}

#[macro_export]
macro_rules! define_plugin {
    (
        name: $name:literal,
        description: $description:literal,
        id: $id:literal,
        actions: $actions:expr,
        variables: $variables:expr,
        data: $data:expr
    ) => {
        const __NAME: &str = concat!($name, "\0");
        const __DESCRIPTION: &str = concat!($description, "\0");
        const __ID: &str = concat!($id, "\0");
        const __ACTIONS: &str = concat!($actions, "\0");
        const __VARIABLES: &str = concat!($variables, "\0");

        #[unsafe(no_mangle)]
        pub extern "C" fn get_name() -> *const ::std::ffi::c_char {
            __NAME.as_ptr() as _
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn get_description() -> *const ::std::ffi::c_char {
            __DESCRIPTION.as_ptr() as _
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn get_id() -> *const ::std::ffi::c_char {
            __ID.as_ptr() as _
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn get_actions() -> *const ::std::ffi::c_char {
            __ACTIONS.as_ptr() as _
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn get_variables() -> *const ::std::ffi::c_char {
            __VARIABLES.as_ptr() as _
        }

        #[unsafe(no_mangle)]
        static PLUGIN: $crate::CPlugin = $data;
    };
}
