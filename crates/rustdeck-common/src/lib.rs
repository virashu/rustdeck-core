use std::ffi::{c_char, c_void};

pub mod prelude {
    pub use super::{Action, ActionArg, Plugin, Variable};
}

pub enum Type {
    Bool,
    Int,
    Float,
    String,
}

impl From<Type> for i32 {
    fn from(val: Type) -> Self {
        match val {
            Type::Bool => 0,
            Type::Int => 1,
            Type::Float => 2,
            Type::String => 3,
        }
    }
}

impl TryFrom<i32> for Type {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Bool,
            1 => Self::Int,
            2 => Self::Float,
            3 => Self::String,
            _ => panic!("Invalid type value"),
        })
    }
}

impl TryFrom<&str> for Type {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "bool" => Self::Bool,
            "int" => Self::Int,
            "float" => Self::Float,
            "string" => Self::String,
            _ => panic!("Invalid type value"),
        })
    }
}

#[repr(C)]
pub struct ActionArg {
    pub id: *const c_char,
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

    pub fn_init: unsafe extern "C" fn() -> *mut c_void,
    pub fn_update: unsafe extern "C" fn(state: *mut c_void),
    pub fn_get_variable: unsafe extern "C" fn(state: *mut c_void, id: *const c_char) -> *mut c_char,
    pub fn_run_action: unsafe extern "C" fn(state: *mut c_void, id: *const c_char),
}

pub type BuildFn = unsafe extern "C" fn() -> *const Plugin;

pub mod util {
    use std::{
        ffi::{CStr, CString, c_char},
        mem::ManuallyDrop,
        str::Utf8Error,
    };

    pub unsafe fn ptr_to_str<'a>(ptr: *const c_char) -> &'a str {
        unsafe { CStr::from_ptr(ptr).to_str().unwrap() }
    }

    pub unsafe fn try_ptr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, Utf8Error> {
        unsafe { CStr::from_ptr(ptr).to_str() }
    }

    pub unsafe fn str_to_ptr(s: impl AsRef<str>) -> *const c_char {
        let p = ManuallyDrop::new(CString::new(s.as_ref()));
        p.as_ref().unwrap().as_ptr()
    }
}

// TODO: make some fields optional
#[macro_export]
macro_rules! decl_plugin {
    /* With actions and variables */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        variables: $variables:expr,
        actions: $actions:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        unsafe {
            unsafe extern "C" fn fn_init() -> *mut ::std::ffi::c_void {
                let mut user_state = ($user_fn_init)();
                let mut state = ::std::mem::ManuallyDrop::new(Box::new(user_state));
                (&raw mut (**state)).cast()
            }
            unsafe extern "C" fn fn_update(state: *mut ::std::ffi::c_void) {
                let user_state = unsafe { &mut *state.cast() };
                ($user_fn_update)(user_state);
            }
            unsafe extern "C" fn fn_get_variable(
                state: *mut ::std::ffi::c_void,
                id: *const ::std::ffi::c_char,
            ) -> *mut ::std::ffi::c_char {
                let user_state = unsafe { &mut *state.cast() };
                let id = unsafe { ::std::ffi::CStr::from_ptr(id).to_str().unwrap() };
                let res = ($user_fn_get_variable)(user_state, id);

                return (*::std::mem::ManuallyDrop::new(::std::boxed::Box::new(
                    ::std::ffi::CString::new(res).unwrap(),
                )))
                .as_ptr()
                .cast_mut();
            }
            unsafe extern "C" fn fn_run_action(
                state: *mut ::std::ffi::c_void,
                id: *const ::std::ffi::c_char,
            ) {
                let user_state = unsafe { &mut *state.cast() };
                let id = unsafe { ::std::ffi::CStr::from_ptr(id).to_str().unwrap() };
                ($user_fn_run_action)(user_state, id);
            }

            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::Plugin {
                id: $crate::util::str_to_ptr($id),
                name: $crate::util::str_to_ptr($name),
                desc: $crate::util::str_to_ptr($desc),
                variables: $variables,
                actions: $actions,

                fn_init: fn_init,
                fn_update: fn_update,
                fn_get_variable: fn_get_variable,
                fn_run_action: fn_run_action,
            })) as *const $crate::Plugin
        }
    };

    /* Without actions nor variables */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: ::std::ptr::null(),
            actions: ::std::ptr::null(),
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action
        }
    };

    /* With variables */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        variables: $variables:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: $variables,
            actions: ::std::ptr::null(),
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action
        }
    };

    /* With actions */
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        actions: $actions:expr,

        fn_init: $user_fn_init:expr,
        fn_update: $user_fn_update:expr,
        fn_get_variable: $user_fn_get_variable:expr,
        fn_run_action: $user_fn_run_action:expr

        $(,)?
    ) => {
        decl_plugin! {
            id: $id,
            name: $name,
            desc: $desc,
            variables: ::std::ptr::null(),
            actions: $actions,
            fn_init: $user_fn_init,
            fn_update: $user_fn_update,
            fn_get_variable: $user_fn_get_variable,
            fn_run_action: $user_fn_run_action
        }
    };
}

#[macro_export]
macro_rules! variables {
    (
        $($var:expr),+ $(,)?
    ) => {
        unsafe {
            ::std::mem::ManuallyDrop::new(vec![
                $($var,)+
                ::std::ptr::null()
            ]).as_ptr() as *const *const $crate::Variable
        }
    };
}

#[macro_export]
macro_rules! decl_variable {
    (
        id: $id:literal,
        desc: $desc:literal,
        vtype: $vtype:literal
        $(,)?
    ) => {
        unsafe {
            ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::Variable {
                id: $crate::util::str_to_ptr($id),
                desc: $crate::util::str_to_ptr($desc),
                r#type: $crate::Type::try_from($vtype)
                    .expect("Incorrect variable type")
                    .into(),
            })) as *const $crate::Variable
        }
    };
}

#[macro_export]
macro_rules! actions {
    (
        $($act:expr),+ $(,)?
    ) => {
        unsafe {
            ::std::mem::ManuallyDrop::new(vec![
                $($act,)+
                ::std::ptr::null()
            ]).as_ptr() as *const *const $crate::Action
        }
    };
}

#[macro_export]
macro_rules! decl_action {
    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal,
        args: $args:expr
        $(,)?
    ) => {
        ::std::boxed::Box::into_raw(::std::boxed::Box::new($crate::Action {
            id: $crate::util::str_to_ptr($id),
            name: $crate::util::str_to_ptr($name),
            desc: $crate::util::str_to_ptr($desc),
            args: $args,
        })) as *const $crate::Action
    };

    (
        id: $id:literal,
        name: $name:literal,
        desc: $desc:literal
        $(,)?
    ) => {
        decl_action! {
            id: $id,
            name: $name,
            desc: $desc,
            args: ::std::ptr::null(),
        }
    };
}
