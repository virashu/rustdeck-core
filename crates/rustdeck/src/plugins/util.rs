use std::ffi::{CString, c_char};

pub unsafe fn read_drop_pointer(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe { CString::from_raw(ptr) }.into_string().unwrap()
}
