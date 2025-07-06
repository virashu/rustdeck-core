use std::ffi::{CString, c_void};

#[repr(C)]
pub struct Result {
    /// 0 = Ok, 1 = Error
    pub status: i32,
    pub content: *mut c_void,
}

#[allow(clippy::fallible_impl_from)]
impl<E: ToString> From<std::result::Result<String, E>> for Result {
    fn from(value: std::result::Result<String, E>) -> Self {
        match value {
            Ok(value) => Self {
                status: 0,
                content: ::std::ffi::CString::new(value).unwrap().into_raw().cast(),
            },
            Err(e) => Self {
                status: 1,
                content: ::std::ffi::CString::new(e.to_string())
                    .unwrap()
                    .into_raw()
                    .cast(),
            },
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl<T, E: ToString> From<std::result::Result<*mut T, E>> for Result {
    fn from(value: std::result::Result<*mut T, E>) -> Self {
        match value {
            Ok(value) => Self {
                status: 0,
                content: value.cast(),
            },
            Err(e) => Self {
                status: 1,
                content: ::std::ffi::CString::new(e.to_string())
                    .unwrap()
                    .into_raw()
                    .cast(),
            },
        }
    }
}

impl From<()> for Result {
    fn from((): ()) -> Self {
        Self {
            status: 0,
            content: std::ptr::null_mut(),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl<T, E: ToString> From<std::result::Result<T, E>> for Result {
    default fn from(value: std::result::Result<T, E>) -> Self {
        match value {
            Ok(value) => Self {
                status: 0,
                content: Box::into_raw(Box::new(value)).cast(),
            },
            Err(e) => Self {
                status: 1,
                content: CString::new(e.to_string()).unwrap().into_raw().cast(),
            },
        }
    }
}
