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
pub struct Plugin {
    pub id: *const c_char,
    pub name: *const c_char,
    pub desc: *const c_char,

    pub variables: *const *const Variable,
    pub actions: *const *const Action,

    pub fn_init: unsafe extern "C" fn() -> Result,
    pub fn_update: unsafe extern "C" fn(state: *mut c_void),
    pub fn_get_variable: unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> Result,
    pub fn_run_action:
        unsafe extern "C" fn(state: *mut c_void, id: *const c_char, args: *const Arg),

    pub fn_get_enum:
        *const unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> *mut c_char,
}

pub type BuildFn = unsafe extern "C" fn() -> *const Plugin;

pub type FreeStringFn = unsafe extern "C" fn(*mut c_char);
