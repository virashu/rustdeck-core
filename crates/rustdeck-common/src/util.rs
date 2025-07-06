use std::{
    ffi::{CStr, CString, c_char},
    str::Utf8Error,
};

/// # Panics
/// Panics because of UTF8 error.
/// # Safety
/// The pointer should be non-null. Use `try_ptr_to_str` for additional check.
#[must_use]
pub unsafe fn ptr_to_str<'a>(ptr: *const c_char) -> &'a str {
    unsafe { CStr::from_ptr(ptr).to_str().unwrap() }
}

#[derive(Debug)]
pub enum PtrToStrError {
    NullPtrError,
    Utf8Error(Utf8Error),
}

impl std::fmt::Display for PtrToStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullPtrError => write!(f, "Error reading pointer: Pointer is Null"),
            Self::Utf8Error(e) => write!(f, "Error reading pointer: {e}"),
        }
    }
}

/// # Safety
/// Checks for null pointer
/// # Errors
/// Returns a `PtrToStrError::NullPtrError` if pointer is null, or `PtrToStrError::Utf8Error` if `CStr::to_str()` method fails
pub unsafe fn try_ptr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, PtrToStrError> {
    if ptr.is_null() {
        return Err(PtrToStrError::NullPtrError);
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map_err(PtrToStrError::Utf8Error)
    }
}

/// # Panics
/// Panics if supplied string has any 0-bytes.
/// Needs to be manually dropped.
pub fn str_to_ptr(s: impl AsRef<str>) -> *mut c_char {
    CString::new(s.as_ref()).unwrap().into_raw()
}
