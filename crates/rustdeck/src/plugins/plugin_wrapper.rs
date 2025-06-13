use libloading::{Library, Symbol};
use rustdeck_common::{CPlugin, interface};

use std::ffi::{CStr, CString, OsStr, c_char, c_void};
use std::fmt::Debug;

use super::error::PluginLoadError;

unsafe fn get_str<'a>(library: &'a Library, ident: &[u8]) -> Result<&'a str, PluginLoadError> {
    // First, the string exported by the plugin is read. For FFI-safety and
    // thread-safety, this must be a function that returns `*const c_char`.
    let name_fn = unsafe { library.get::<extern "C" fn() -> *const c_char>(ident) }?;
    let name: *const c_char = name_fn();

    // Unfortunately there is no way to make sure this part is safe. We have
    // to assume the address exported by the plugin is valid. Otherwise,
    // this part may cause an abort.
    let name = unsafe { CStr::from_ptr(name) };

    // Finally, the string is converted to UTF-8 and returned
    Ok(name.to_str()?)
}

unsafe fn read_drop_pointer(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let c_str: &CStr = unsafe { CStr::from_ptr(ptr) };
    let str_slice: &str = c_str.to_str().unwrap();
    let string = str_slice.to_owned();

    unsafe { std::ptr::drop_in_place(ptr) };

    string
}

pub struct Plugin {
    #[allow(dead_code, reason = "WIP")]
    pub name: String,
    #[allow(dead_code, reason = "WIP")]
    pub description: String,
    pub id: String,
    pub actions: Vec<String>,
    pub variables: Vec<String>,

    inner: CPlugin,
    state: *mut c_void,

    #[allow(dead_code, reason = "plugin depends on library")]
    lib: Library,
}

impl Plugin {
    pub fn try_load<P>(path: P) -> Result<Self, PluginLoadError>
    where
        P: AsRef<OsStr> + Debug,
    {
        unsafe {
            let lib = Library::new(path)?;

            let id = get_str(&lib, interface::ID_IDENT)?
                .to_owned()
                .to_lowercase();
            if id == "deck" {
                return Err(PluginLoadError::FormatError(
                    "Plugin id can not be 'deck', as it is reserved".into(),
                ));
            }

            let name = get_str(&lib, interface::NAME_IDENT)?.to_owned();
            let description = get_str(&lib, interface::DESCRIPTION_IDENT)?.to_owned();
            let actions = get_str(&lib, interface::ACTIONS)?
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect();
            let variables = get_str(&lib, interface::VARIABLES)?
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect();

            let plugin_data = lib
                .get::<Symbol<*const CPlugin>>(interface::PLUGIN_IDENT)
                .unwrap()
                .read();

            let state = (plugin_data.init)();

            Ok(Self {
                name,
                description,
                id,
                actions,
                variables,
                inner: plugin_data,
                state,
                lib,
            })
        }
    }

    pub fn update(&mut self) {
        unsafe { (self.inner.update)(self.state) }
    }

    pub fn run_action(&self, id: String) {
        unsafe {
            (self.inner.run_action)(
                self.state,
                CString::new(id).unwrap().as_ptr().cast::<c_char>(),
            );
        }
    }

    pub fn get_variable<T>(&self, id: T) -> String
    where
        T: AsRef<str>,
    {
        unsafe {
            let p = (self.inner.get_variable)(
                self.state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            );
            read_drop_pointer(p)
        }
    }
}

unsafe impl Send for Plugin {}

unsafe impl Sync for Plugin {}
