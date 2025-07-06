use std::{
    ffi::{CString, OsStr, c_char, c_void},
    fmt::Debug,
    mem::ManuallyDrop,
};

use libloading::Library;
use rustdeck_common::{
    proto::{Arg, BuildFn, FreeStringFn, Plugin as FFIPlugin},
    util::{self, try_ptr_to_str},
};

use super::{
    datatype::PluginDataType,
    error::PluginLoadError,
    proto::{Action, ActionArg, ConfigOption, Variable},
};
use crate::{
    constants::DECK_ACTION_ID,
    plugins::{error::ActionError, safe_arg::SafeArg},
};

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
    pub config_options: Vec<ConfigOption>,

    inner: &'static FFIPlugin,
    state: Option<*mut c_void>,
    has_free: bool,

    lib: Option<Library>,
}

impl Plugin {
    pub fn try_load<P>(path: P) -> Result<Self, PluginLoadError>
    where
        P: AsRef<OsStr> + Debug,
    {
        unsafe {
            let lib = Library::new(path)?;

            let build: libloading::Symbol<BuildFn> = lib.get(b"build")?;
            let ptr = build();
            let mut plugin = Self::try_from_ptr(ptr)?;

            // Test for `free` function
            let has_free = lib.get::<libloading::Symbol<FreeStringFn>>(b"free").is_ok();
            plugin.has_free = has_free;
            plugin.lib = Some(lib);

            Ok(plugin)
        }
    }

    pub fn try_from_ptr(ptr: *const FFIPlugin) -> Result<Self, PluginLoadError> {
        unsafe {
            let plugin = ptr.as_ref().ok_or(PluginLoadError::BuildError)?;

            let id = util::try_ptr_to_str(plugin.id)?.to_owned();

            if id == DECK_ACTION_ID {
                return Err(PluginLoadError::FormatError(
                    "Plugin id can not be 'deck', as it is reserved".into(),
                ));
            }

            let name = util::try_ptr_to_str(plugin.name)?.to_owned();
            let description = util::try_ptr_to_str(plugin.desc)?.to_owned();

            let variables = if plugin.variables.is_null() {
                Vec::new()
            } else {
                (0..isize::MAX)
                    .map_while(|offset| plugin.variables.offset(offset).as_ref().unwrap().as_ref())
                    .map(Variable::from_ffi_ref)
                    .collect::<Result<Vec<Variable>, PluginLoadError>>()?
            };

            let actions = if plugin.actions.is_null() {
                Vec::new()
            } else {
                (0..isize::MAX)
                    .map_while(|offset| plugin.actions.offset(offset).as_ref().unwrap().as_ref())
                    .map(Action::from_ffi_ref)
                    .collect::<Result<Vec<Action>, PluginLoadError>>()?
            };

            let config_options = if plugin.config_options.is_null() {
                Vec::new()
            } else {
                (0..isize::MAX)
                    .map_while(|offset| {
                        plugin
                            .config_options
                            .offset(offset)
                            .as_ref()
                            .unwrap()
                            .as_ref()
                    })
                    .map(ConfigOption::from_ffi_ref)
                    .collect::<Result<Vec<ConfigOption>, PluginLoadError>>()?
            };

            Ok(Self {
                name,
                description,
                id,
                actions,
                variables,
                config_options,
                inner: plugin,
                has_free: false,
                state: None,
                lib: None,
            })
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        unsafe {
            let state_res = (self.inner.fn_init)();

            let state = if state_res.status == 0 {
                state_res.content
            } else {
                return Err(try_ptr_to_str(state_res.content.cast()).map_or_else(
                    |_| String::new(),
                    |error| {
                        let error = error.to_owned();
                        self.free(state_res.content.cast());
                        error
                    },
                ));
            };

            self.state = Some(state);
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), String> {
        let state = self.state.expect("Plugin is not initialized");

        unsafe {
            let res = (self.inner.fn_update)(state);

            if res.status == 0 {
                Ok(())
            } else {
                try_ptr_to_str(res.content.cast()).map_or_else(
                    |_| Err(String::from("<no error description>")),
                    |error| {
                        let error = error.to_owned();
                        self.free(res.content.cast());
                        Err(error)
                    },
                )
            }
        }
    }

    fn free(&self, ptr: *mut c_char) {
        if !self.has_free {
            return;
        }

        unsafe {
            let free: libloading::Symbol<FreeStringFn> =
                self.lib.as_ref().unwrap().get(b"free").unwrap();
            free(ptr);
        }
    }

    /// Validate args and run action
    pub fn run_action(&self, act_id: String, args: &[String]) -> Result<(), ActionError> {
        let state = self.state.expect("Plugin is not initialized");

        let Some(action_prototype) = self.actions.iter().find(|v| v.id == act_id) else {
            return Err(ActionError::ActionNotFound {
                plugin: self.id.clone(),
                action: act_id,
            });
        };

        let safe_args = Self::parse_args(&action_prototype.args, args)
            .map_err(|_| ActionError::InvalidArgs(act_id.clone()))?;

        unsafe {
            let res = (self.inner.fn_run_action)(
                state,
                CString::new(act_id).unwrap().as_ptr().cast::<c_char>(),
                safe_args
                    .iter()
                    .map(super::safe_arg::SafeArg::as_arg)
                    .collect::<Vec<Arg>>()
                    .as_ptr(),
            );

            if res.status != 0 {
                let error_value = util::try_ptr_to_str(res.content.cast()).map_or_else(
                    |_| String::from("<no error description>"),
                    ToOwned::to_owned,
                );

                return Err(ActionError::PluginError(error_value));
            }
        }

        Ok(())
    }

    pub fn get_variable<T>(&self, id: T) -> Result<String, String>
    where
        T: AsRef<str>,
    {
        let state = self.state.expect("Plugin is not initialized");

        unsafe {
            let res = (self.inner.fn_get_variable)(
                state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            );

            if res.status == 0 {
                let value = try_ptr_to_str(res.content.cast()).unwrap().to_owned();
                self.free(res.content.cast());
                Ok(value)
            } else {
                try_ptr_to_str(res.content.cast()).map_or_else(
                    |_| Err(String::from("<no error description>")),
                    |error| {
                        let error = error.to_owned();
                        self.free(res.content.cast());
                        Err(error)
                    },
                )
            }
        }
    }

    pub fn parse_args(
        proto: &[ActionArg],
        args: &[String],
    ) -> Result<Vec<SafeArg>, Box<dyn std::error::Error>> {
        if proto.len() != args.len() {
            return Err("Argument list length doesn't match".into());
        }

        let mut parsed = Vec::with_capacity(proto.len());

        for (pr, arg_str) in proto.iter().zip(args) {
            let arg = match pr.r#type {
                PluginDataType::Bool => SafeArg::Bool(Arg {
                    b: Box::into_raw(Box::new(arg_str.parse::<bool>()?)),
                }),
                PluginDataType::Int => SafeArg::Int(Arg {
                    i: Box::into_raw(Box::new(arg_str.parse::<i32>()?)),
                }),
                PluginDataType::Float => SafeArg::Float(Arg {
                    f: Box::into_raw(Box::new(arg_str.parse::<f32>()?)),
                }),
                PluginDataType::String => SafeArg::String(Arg {
                    c: ManuallyDrop::new(CString::new(arg_str.clone())?)
                        .as_ptr()
                        .cast::<c_char>(),
                }),
                PluginDataType::Enum => SafeArg::String(Arg {
                    c: ManuallyDrop::new(CString::new(arg_str.clone())?)
                        .as_ptr()
                        .cast::<c_char>(),
                }),
            };
            parsed.push(arg);
        }

        Ok(parsed)
    }

    pub fn get_enum_arg<T>(&self, id: T) -> Result<Vec<String>, String>
    where
        T: AsRef<str>,
    {
        let state = self.state.expect("Plugin is not initialized");

        unsafe {
            let res = (self.inner.fn_get_enum.as_ref().unwrap())(
                state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            );

            if res.status == 0 {
                let value = try_ptr_to_str(res.content.cast()).unwrap().to_owned();
                self.free(res.content.cast());
                Ok(value.split('\n').map(ToOwned::to_owned).collect())
            } else {
                try_ptr_to_str(res.content.cast()).map_or_else(
                    |_| Err(String::from("<no error description>")),
                    |error| {
                        let error = error.to_owned();
                        self.free(res.content.cast());
                        Err(error)
                    },
                )
            }
        }
    }

    pub fn get_config_value<T>(&mut self, id: T) -> Result<String, String>
    where
        T: AsRef<str>,
    {
        let state = self.state.expect("Plugin is not initialized");

        unsafe {
            (self.inner.fn_get_config_value.as_ref().unwrap())(
                state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            );
        }

        todo!()
    }

    pub fn set_config_value<T>(&mut self, id: T, value: String) -> Result<(), String>
    where
        T: AsRef<str>,
    {
        let state = self.state.expect("Plugin is not initialized");

        let arg = SafeArg::String(Arg {
            c: ManuallyDrop::new(CString::new(value).unwrap())
                .as_ptr()
                .cast::<c_char>(),
        });
        unsafe {
            (self.inner.fn_set_config_value.as_ref().unwrap())(
                state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
                std::ptr::from_ref(&arg.as_arg()),
            );
        }
        Ok(())
    }
}

unsafe impl Send for Plugin {}

unsafe impl Sync for Plugin {}

#[cfg(test)]
mod tests {
    use rustdeck_common::{
        Args, actions, args, decl_action, decl_arg, decl_plugin, decl_variable, variables,
    };

    use super::*;

    struct PluginState {
        counter: i32,
    }

    #[allow(clippy::unnecessary_wraps)]
    fn init() -> Result<PluginState, Box<dyn std::error::Error>> {
        Ok(PluginState { counter: 0 })
    }

    fn update(_: &PluginState) {}

    fn get_variable(state: &PluginState, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        if id == "counter" {
            Ok(state.counter.to_string())
        } else {
            unreachable!()
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn run_action(
        state: &mut PluginState,
        id: &str,
        args: &Args,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match id {
            "increment" => state.counter += 1,
            "add" => {
                let amt = args.get(0).int();
                state.counter += amt;
            }
            "print" => {
                let a = args.get(0);
                let s = a.string();
                println!("{s}");
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    fn build() -> *const FFIPlugin {
        decl_plugin! {
            id: "test_plugin",
            name: "Test Plugin",
            desc: "Test Plugin",
            variables: variables!(
                decl_variable! {
                    id: "counter",
                    desc: "Counter",
                    vtype: "int",
                },
            ),
            actions: actions!(
                decl_action! {
                    id: "increment",
                    name: "Increment",
                    desc: "Increment",
                },
                decl_action! {
                    id: "add",
                    name: "Add",
                    desc: "Add",
                    args: args!(
                        decl_arg! {
                            id: "amount",
                            name: "Amount",
                            desc: "Amount",
                            vtype: "int",
                        },
                    ),
                },
                decl_action! {
                    id: "print",
                    name: "Print",
                    desc: "Print",
                    args: args!(
                        decl_arg! {
                            id: "string",
                            name: "String",
                            desc: "String",
                            vtype: "string",
                        },
                    ),
                }
            ),
            fn_init: init,
            fn_update: update,
            fn_get_variable: get_variable,
            fn_run_action: run_action,
        }
    }

    #[test]
    fn test_plugin() {
        let plugin = Plugin::try_from_ptr(build());
        assert!(plugin.is_ok());
        let plugin = plugin.unwrap();

        assert_eq!(plugin.get_variable("counter").unwrap(), "0");

        assert!(plugin.run_action("increment".into(), &[]).is_ok());
        assert_eq!(plugin.get_variable("counter").unwrap(), "1");

        assert!(plugin.run_action("add".into(), &["10".into()]).is_ok());
        assert_eq!(plugin.get_variable("counter").unwrap(), "11");

        assert!(
            plugin
                .run_action("print".into(), &["Hello!".into()])
                .is_ok()
        );
    }
}
