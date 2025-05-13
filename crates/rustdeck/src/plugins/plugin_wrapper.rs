use libloading::{Library, Symbol};
use std::{
    error::Error,
    ffi::{c_char, c_void, CStr, CString, OsStr},
    fmt::Debug,
};

use rustdeck_common::{interface, CPlugin};

use crate::error::report_libloading_error;

unsafe fn get_str<'a>(
    library: &'a Library,
    ident: &[u8],
) -> Result<&'a str, Box<dyn std::error::Error>> {
    // First, the string exported by the plugin is read. For FFI-safety and
    // thread-safety, this must be a function that returns `*const c_char`.
    let name_fn = library.get::<extern "C" fn() -> *const c_char>(ident)?;
    let name: *const c_char = name_fn();

    // Unfortunately there is no way to make sure this part is safe. We have
    // to assume the address exported by the plugin is valid. Otherwise,
    // this part may cause an abort.
    let name = CStr::from_ptr(name);

    // Finally, the string is converted to UTF-8 and returned
    Ok(name.to_str()?)
}

unsafe fn read_drop_pointer(ptr: *mut c_char) -> String {
    let c_str: &CStr = CStr::from_ptr(ptr);
    let str_slice: &str = c_str.to_str().unwrap();
    let string = str_slice.to_owned();

    std::ptr::drop_in_place(ptr);

    string
}

pub struct Plugin {
    pub name: String,
    pub description: String,
    pub id: String,
    pub actions: Vec<String>,
    pub variables: Vec<String>,

    plugin_data: CPlugin,
    state: *mut c_void,

    #[allow(dead_code, reason = "`plugin` depends on `lib`")]
    lib: Library,
}

impl Plugin {
    pub fn try_load<P: AsRef<OsStr> + Debug>(path: P) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let lib = Library::new(path).inspect_err(report_libloading_error)?;

            let name = get_str(&lib, interface::NAME_IDENT)?.to_owned();
            let description = get_str(&lib, interface::DESCRIPTION_IDENT)?.to_owned();
            let id = get_str(&lib, interface::ID_IDENT)?.to_owned();
            let actions = get_str(&lib, interface::ACTIONS)?
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned())
                .collect();
            let variables = get_str(&lib, interface::VARIABLES)?
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned())
                .collect();

            let plugin_data = lib
                .get::<Symbol<*const CPlugin>>(interface::PLUGIN_IDENT)
                .unwrap()
                .read();

            let state = (plugin_data.init)();

            Ok(Self {
                name,
                id,
                description,
                actions,
                variables,
                plugin_data,
                state,
                lib,
            })
        }
    }

    pub fn update(&mut self) {
        unsafe { (self.plugin_data.update)(self.state) }
    }

    pub fn run_action(&self, id: String) {
        unsafe {
            (self.plugin_data.run_action)(
                self.state,
                CString::new(id).unwrap().as_ptr() as *const c_char,
            )
        }
    }

    pub fn get_variable(&self, id: String) -> String {
        unsafe {
            let p = (self.plugin_data.get_variable)(
                self.state,
                CString::new(id).unwrap().as_ptr() as *const c_char,
            );
            read_drop_pointer(p)
        }
    }
}

unsafe impl Send for Plugin {}
