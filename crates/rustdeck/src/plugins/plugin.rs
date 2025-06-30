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

use super::{datatype::PluginDataType, error::PluginLoadError};
use crate::{
    constants::DECK_ACTION_ID,
    plugins::{error::ActionError, safe_arg::SafeArg},
};

/// Args are positional
#[derive(Clone)]
pub struct ActionArg {
    pub id: String,
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
    has_free: bool,

    #[allow(dead_code, reason = "plugin depends on library")]
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
                                id: util::try_ptr_to_str(arg.id)?.to_owned(),
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
                has_free: false,
                state,
                lib: None,
            })
        }
    }

    pub fn update(&mut self) {
        unsafe { (self.inner.fn_update)(self.state) }
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
        let Some(action_prototype) = self.actions.iter().find(|v| v.id == act_id) else {
            return Err(ActionError::ActionNotFound {
                plugin: self.id.clone(),
                action: act_id,
            });
        };

        let safe_args = Self::parse_args(&action_prototype.args, args)
            .map_err(|_| ActionError::InvalidArgs(act_id.clone()))?;

        unsafe {
            (self.inner.fn_run_action)(
                self.state,
                CString::new(act_id).unwrap().as_ptr().cast::<c_char>(),
                safe_args
                    .iter()
                    .map(super::safe_arg::SafeArg::as_arg)
                    .collect::<Vec<Arg>>()
                    .as_ptr(),
            );
        }

        Ok(())
    }

    pub fn get_variable<T>(&self, id: T) -> String
    where
        T: AsRef<str>,
    {
        unsafe {
            let res_ptr = (self.inner.fn_get_variable)(
                self.state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            );
            let res = try_ptr_to_str(res_ptr).unwrap().to_owned();

            self.free(res_ptr);

            res
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
                PluginDataType::Enum => todo!(),
            };
            parsed.push(arg);
        }

        Ok(parsed)
    }

    pub fn get_enum_arg<T>(&self, id: T) -> Vec<String>
    where
        T: AsRef<str>,
    {
        let res_ptr = unsafe {
            (self.inner.fn_get_enum.as_ref().unwrap())(
                self.state,
                CString::new(id.as_ref()).unwrap().as_ptr().cast::<c_char>(),
            )
        };

        let res = unsafe { try_ptr_to_str(res_ptr).unwrap().to_owned() };

        self.free(res_ptr);

        res.split('\n').map(ToOwned::to_owned).collect()
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

    fn init() -> PluginState {
        PluginState { counter: 0 }
    }

    fn update(_: &PluginState) {}

    fn get_variable(state: &PluginState, id: &str) -> String {
        if id == "counter" {
            state.counter.to_string()
        } else {
            unreachable!()
        }
    }

    fn run_action(state: &mut PluginState, id: &str, args: &Args) {
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
    }

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

        assert_eq!(plugin.get_variable("counter"), "0");

        assert!(plugin.run_action("increment".into(), &[]).is_ok());
        assert_eq!(plugin.get_variable("counter"), "1");

        assert!(plugin.run_action("add".into(), &["10".into()]).is_ok());
        assert_eq!(plugin.get_variable("counter"), "11");

        assert!(
            plugin
                .run_action("print".into(), &["Hello!".into()])
                .is_ok()
        );
    }
}
