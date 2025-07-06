use std::ffi::{c_char, c_void};

#[repr(C)]
pub struct Result {
    /// 0 = Ok, 1 = Error
    pub status: i32,
    pub content: *mut c_void,
}

#[repr(C)]
pub union Arg {
    pub b: *const bool,
    pub i: *const i32,
    pub f: *const f32,
    pub c: *const c_char,
}

#[repr(C)]
pub struct ActionArg {
    pub id: *const c_char,
    pub name: *const c_char,
    pub desc: *const c_char,
    pub r#type: i32,
}

#[repr(C)]
pub struct Action {
    pub id: *const c_char,
    pub name: *const c_char,
    pub desc: *const c_char,
    pub args: *const *const ActionArg,
}

#[repr(C)]
pub struct Variable {
    pub id: *const c_char,
    pub desc: *const c_char,
    pub r#type: i32,
}

#[repr(C)]
pub struct ConfigOption {
    pub id: *const c_char,
    pub name: *const c_char,
    pub desc: *const c_char,
    pub r#type: i32,
}

/* Methods */
pub type FnInit = unsafe extern "C" fn() -> Result;
pub type FnUpdate = unsafe extern "C" fn(state: *mut c_void);
pub type FnGetVariable = unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> Result;
pub type FnRunAction =
    unsafe extern "C" fn(state: *mut c_void, id: *const c_char, args: *const Arg) -> Result;
pub type FnGetEnum = *const unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> Result;
pub type FnGetConfigValue =
    *const unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> Result;

#[repr(C)]
pub struct Plugin {
    pub id: *const c_char,
    pub name: *const c_char,
    pub desc: *const c_char,

    pub variables: *const *const Variable,
    pub actions: *const *const Action,
    pub config_options: *const *const ConfigOption,

    pub fn_init: FnInit,
    pub fn_update: FnUpdate,
    pub fn_get_variable: FnGetVariable,
    pub fn_run_action: FnRunAction,

    /* Optional */
    pub fn_get_enum: FnGetEnum,
    pub fn_get_config_value: FnGetConfigValue,
}

/* Globals */
pub type BuildFn = unsafe extern "C" fn() -> *const Plugin;
pub type FreeStringFn = unsafe extern "C" fn(*mut c_char);
