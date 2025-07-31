use std::ffi::c_void;

use crate::util;

#[repr(C)]
pub struct Result {
    /// 0 = Ok, 1 = Error
    pub status: i32,
    pub content: *mut c_void,
}

impl Result {
    pub fn from_string_result<E: ToString>(value: std::result::Result<String, E>) -> Self {
        match value {
            Ok(value) => Self {
                status: 0,
                content: util::str_to_ptr(value).cast(),
            },
            Err(e) => Self {
                status: 1,
                content: util::str_to_ptr(e.to_string()).cast(),
            },
        }
    }

    pub fn from_ptr_result<T, E: ToString>(value: std::result::Result<*mut T, E>) -> Self {
        match value {
            Ok(value) => Self {
                status: 0,
                content: value.cast(),
            },
            Err(e) => Self {
                status: 1,
                content: util::str_to_ptr(e.to_string()).cast(),
            },
        }
    }

    pub fn from_any_result<T, E: ToString>(value: std::result::Result<T, E>) -> Self {
        match value {
            Ok(value) => Self {
                status: 0,
                content: Box::into_raw(Box::new(value)).cast(),
            },
            Err(e) => Self {
                status: 1,
                content: util::str_to_ptr(e.to_string()).cast(),
            },
        }
    }
}

impl Default for Result {
    fn default() -> Self {
        Self {
            status: 0,
            content: std::ptr::null_mut(),
        }
    }
}

impl<E: ToString> From<std::result::Result<String, E>> for Result {
    fn from(value: std::result::Result<String, E>) -> Self {
        Self::from_string_result(value)
    }
}

impl<T, E: ToString> From<std::result::Result<*mut T, E>> for Result {
    fn from(value: std::result::Result<*mut T, E>) -> Self {
        Self::from_ptr_result(value)
    }
}

impl From<()> for Result {
    fn from((): ()) -> Self {
        Self::default()
    }
}

#[allow(clippy::fallible_impl_from)]
impl<T, E: ToString> From<std::result::Result<T, E>> for Result {
    default fn from(value: std::result::Result<T, E>) -> Self {
        Self::from_any_result(value)
    }
}
