use libloading::Library;

use std::{
    ffi::{CStr, CString, OsStr, c_char, c_void},
    fmt::Debug,
    mem::ManuallyDrop,
};

use rustdeck_common::{
    proto::{Arg, BuildFn, Plugin as FFIPlugin},
    util,
};

use crate::constants::DECK_ACTION_ID;

use super::error::PluginLoadError;

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

#[derive(Clone)]
pub enum PluginDataType {
    Bool,
    Int,
    Float,
    String,
    Enum,
}

impl TryFrom<i32> for PluginDataType {
    type Error = PluginLoadError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bool),
            1 => Ok(Self::Int),
            2 => Ok(Self::Float),
            3 => Ok(Self::String),
            4 => Ok(Self::Enum),
            _ => Err(PluginLoadError::FormatError(format!(
                "No plugin data type with index '{value}'"
            ))),
        }
    }
}

impl std::fmt::Display for PluginDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bool => "bool",
                Self::Int => "int",
                Self::Float => "float",
                Self::String => "string",
                Self::Enum => "enum",
            }
        )
    }
}

/// Args are positional
#[derive(Clone)]
pub struct ActionArg {
    pub name: String,
    pub description: String,
    pub r#type: PluginDataType,
}

#[derive(Clone)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<ActionArg>,
}

#[derive(Clone)]
pub struct Variable {
    pub id: String,
    pub description: String,
    pub r#type: PluginDataType,
}

/// Wrapper to isolate all the unsafe operations
///
/// Wraps `rustdeck_common::Plugin`
///
/// # Safety
/// User has to lock the struct, as it itself has no mutexes
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub id: String,
    pub actions: Vec<Action>,
    pub variables: Vec<Variable>,

    inner: &'static FFIPlugin,
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
            let build: libloading::Symbol<BuildFn> = lib.get(b"build")?;
            let plugin_raw = build();
            let plugin = plugin_raw.as_ref().ok_or(PluginLoadError::BuildError)?;

            let id = util::try_ptr_to_str(plugin.id)?.to_owned();

            if id == DECK_ACTION_ID {
                return Err(PluginLoadError::FormatError(
                    "Plugin id can not be 'deck', as it is reserved".into(),
                ));
            }

            let name = util::try_ptr_to_str(plugin.name)?.to_owned();
            let description = util::try_ptr_to_str(plugin.desc)?.to_owned();

            let mut variables = Vec::new();
            if !plugin.variables.is_null() {
                let mut vars_offset = 0;
                while let Some(var) = plugin
                    .variables
                    .offset(vars_offset)
                    .as_ref()
                    .unwrap()
                    .as_ref()
                {
                    variables.push(Variable {
                        id: util::try_ptr_to_str(var.id)?.to_owned(),
                        description: util::try_ptr_to_str(var.desc)?.to_owned(),
                        r#type: var.r#type.try_into()?,
                    });
                    vars_offset += 1;
                }
            }

            let mut actions = Vec::new();
            if !plugin.actions.is_null() {
                let mut actions_offset = 0;
                while let Some(act) = plugin
                    .actions
                    .offset(actions_offset)
                    .as_ref()
                    .unwrap()
                    .as_ref()
                {
                    let mut args = Vec::new();

                    if !act.args.is_null() {
                        let mut args_offset = 0;
                        while let Some(arg) =
                            act.args.offset(args_offset).as_ref().unwrap().as_ref()
                        {
                            args.push(ActionArg {
                                name: util::try_ptr_to_str(arg.name)?.to_owned(),
                                description: util::try_ptr_to_str(arg.desc)?.to_owned(),
                                r#type: arg.r#type.try_into()?,
                            });

                            args_offset += 1;
                        }
                    }

                    actions.push(Action {
                        id: util::try_ptr_to_str(act.id)?.to_owned(),
                        name: util::try_ptr_to_str(act.name)?.to_owned(),
                        description: util::try_ptr_to_str(act.desc)?.to_owned(),
                        args,
                    });

                    actions_offset += 1;
                }
            }

            let state = (plugin.fn_init)();

            Ok(Self {
                name,
                description,
                id,
                actions,
                variables,
                inner: plugin,
                state,
                lib,
            })
        }
    }

    pub fn update(&mut self) {
        unsafe { (self.inner.fn_update)(self.state) }
    }

    pub fn run_action(&self, id: String, args: &[Arg]) {
        unsafe {
            (self.inner.fn_run_action)(
                self.state,
                CString::new(id).unwrap().as_ptr().cast::<c_char>(),
                args.as_ptr(),
            );
        }
    }

    pub fn get_variable<T>(&self, id: T) -> String
    where
        T: AsRef<str>,
    {
        unsafe {
            let p = (self.inner.fn_get_variable)(
                self.state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            );
            read_drop_pointer(p)
        }
    }

    pub fn parse_args(proto: &Vec<ActionArg>, args: &[String]) -> Vec<Arg> {
        args.iter()
            .zip(proto)
            .map(|(a, p)| match p.r#type {
                PluginDataType::Bool => Arg {
                    b: &(a.parse::<bool>().unwrap()),
                },
                PluginDataType::Int => Arg {
                    i: Box::into_raw(Box::new(a.parse::<i32>().unwrap())),
                },
                PluginDataType::Float => Arg {
                    f: Box::into_raw(Box::new(a.parse::<f32>().unwrap())),
                },
                PluginDataType::String => Arg {
                    // FIXME: Leak.
                    c: ManuallyDrop::new(CString::new(a.clone()).unwrap())
                        .as_ptr()
                        .cast::<c_char>(),
                },
                PluginDataType::Enum => todo!(),
            })
            .collect()
    }
}

unsafe impl Send for Plugin {}

unsafe impl Sync for Plugin {}
