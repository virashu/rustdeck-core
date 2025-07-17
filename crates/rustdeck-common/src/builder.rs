use std::mem::ManuallyDrop;

use crate::{Type, proto, util};

pub struct Variable {
    pub id: String,
    pub desc: String,
    pub vtype: Type,
}

pub struct ActionArg {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub vtype: Type,
}

pub struct Action {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub args: Vec<ActionArg>,
}

pub struct ConfigOption {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub vtype: Type,
}

impl Variable {
    pub fn new(id: impl Into<String>, desc: impl Into<String>, vtype: impl Into<Type>) -> Self {
        Self {
            id: id.into(),
            desc: desc.into(),
            vtype: vtype.into(),
        }
    }

    #[must_use]
    pub fn build(self) -> *const proto::Variable {
        Box::into_raw(Box::new(proto::Variable {
            id: util::str_to_ptr(self.id),
            desc: util::str_to_ptr(self.desc),
            r#type: self.vtype.into(),
        }))
    }
}

impl Action {
    pub fn new(id: impl Into<String>, name: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            desc: desc.into(),
            args: Vec::new(),
        }
    }

    #[must_use]
    pub fn arg(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        desc: impl Into<String>,
        vtype: impl Into<Type>,
    ) -> Self {
        self.args.push(ActionArg {
            id: id.into(),
            name: name.into(),
            desc: desc.into(),
            vtype: vtype.into(),
        });
        self
    }
}

impl ConfigOption {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        desc: impl Into<String>,
        vtype: impl Into<Type>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            desc: desc.into(),
            vtype: vtype.into(),
        }
    }
}

#[must_use]
pub struct PluginBuilder {
    id: String,
    name: String,
    desc: String,

    variables: Option<Vec<Variable>>,
    actions: Option<Vec<Action>>,
    config_options: Option<Vec<ConfigOption>>,

    fn_init: Option<proto::FnInit>,
    fn_update: Option<proto::FnUpdate>,
    fn_get_variable: Option<proto::FnGetVariable>,
    fn_run_action: Option<proto::FnRunAction>,

    fn_get_enum: Option<*const proto::FnGetEnum>,
    fn_get_config_value: Option<*const proto::FnGetConfigValue>,
    fn_set_config_value: Option<*const proto::FnSetConfigValue>,
}

impl PluginBuilder {
    pub fn new(id: impl AsRef<str>, name: impl AsRef<str>, desc: impl AsRef<str>) -> Self {
        Self {
            id: id.as_ref().to_owned(),
            name: name.as_ref().to_owned(),
            desc: desc.as_ref().to_owned(),

            variables: None,
            actions: None,
            config_options: None,

            fn_init: None,
            fn_update: None,
            fn_get_variable: None,
            fn_run_action: None,

            fn_get_enum: None,
            fn_get_config_value: None,
            fn_set_config_value: None,
        }
    }

    pub fn init(mut self, f: proto::FnInit) -> Self {
        self.fn_init = Some(f);
        self
    }

    pub fn variable(mut self, var: Variable) -> Self {
        if let Some(vars) = &mut self.variables {
            vars.push(var);
        } else {
            self.variables = Some(vec![var]);
        }

        self
    }

    pub fn action(mut self, act: Action) -> Self {
        if let Some(acts) = &mut self.actions {
            acts.push(act);
        } else {
            self.actions = Some(vec![act]);
        }

        self
    }

    pub fn config_option(mut self, opt: ConfigOption) -> Self {
        if let Some(opts) = &mut self.config_options {
            opts.push(opt);
        } else {
            self.config_options = Some(vec![opt]);
        }

        self
    }

    pub fn update(mut self, f: proto::FnUpdate) -> Self {
        self.fn_update = Some(f);
        self
    }

    pub fn get_variable(mut self, f: proto::FnGetVariable) -> Self {
        self.fn_get_variable = Some(f);
        self
    }

    pub fn run_action(mut self, f: proto::FnRunAction) -> Self {
        self.fn_run_action = Some(f);
        self
    }

    pub fn get_enum(mut self, f: *const proto::FnGetEnum) -> Self {
        self.fn_get_enum = Some(f);
        self
    }

    pub fn get_config_value(mut self, f: *const proto::FnGetConfigValue) -> Self {
        self.fn_get_config_value = Some(f);
        self
    }

    pub fn set_config_value(mut self, f: *const proto::FnSetConfigValue) -> Self {
        self.fn_set_config_value = Some(f);
        self
    }

    /// Builds the plugin.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the required functions are not set.
    #[allow(clippy::similar_names)]
    pub fn build(self) -> Result<*const proto::Plugin, String> {
        let fn_init = self
            .fn_init
            .ok_or_else(|| "fn_init is not set".to_string())?;
        let fn_update = self
            .fn_update
            .ok_or_else(|| "fn_update is not set".to_string())?;
        let fn_get_variable = self
            .fn_get_variable
            .ok_or_else(|| "fn_get_variable is not set".to_string())?;
        let fn_run_action = self
            .fn_run_action
            .ok_or_else(|| "fn_run_action is not set".to_string())?;
        let fn_get_enum = self.fn_get_enum.unwrap_or_else(std::ptr::null);

        let fn_get_config_value = self.fn_get_config_value.unwrap_or_else(std::ptr::null);
        let fn_set_config_value = self.fn_set_config_value.unwrap_or_else(std::ptr::null);

        #[allow(clippy::option_if_let_else)]
        let variables = match self.variables {
            Some(vars) => ManuallyDrop::new(
                vars.into_iter()
                    .map(Variable::build)
                    .chain(vec![std::ptr::null()])
                    .collect::<Vec<_>>(),
            )
            .as_ptr(),
            None => std::ptr::null(),
        };

        #[allow(clippy::option_if_let_else)]
        let actions = match self.actions {
            Some(acts) => ManuallyDrop::new(
                acts.into_iter()
                    .map(|act| {
                        Box::into_raw(Box::new(proto::Action {
                            id: util::str_to_ptr(act.id),
                            name: util::str_to_ptr(act.name),
                            desc: util::str_to_ptr(act.desc),
                            args: ManuallyDrop::new(
                                act.args
                                    .into_iter()
                                    .map(|arg| {
                                        Box::into_raw(Box::new(proto::ActionArg {
                                            id: util::str_to_ptr(arg.id),
                                            name: util::str_to_ptr(arg.name),
                                            desc: util::str_to_ptr(arg.desc),
                                            r#type: arg.vtype.into(),
                                        }))
                                        .cast_const()
                                    })
                                    .chain(vec![std::ptr::null()])
                                    .collect::<Vec<_>>(),
                            )
                            .as_ptr(),
                        }))
                        .cast_const()
                    })
                    .chain(vec![std::ptr::null()])
                    .collect::<Vec<_>>(),
            )
            .as_ptr(),
            None => std::ptr::null(),
        };

        #[allow(clippy::option_if_let_else)]
        let config_options = match self.config_options {
            Some(options) => ManuallyDrop::new(
                options
                    .into_iter()
                    .map(|option| {
                        Box::into_raw(Box::new(proto::ConfigOption {
                            id: util::str_to_ptr(option.id),
                            name: util::str_to_ptr(option.name),
                            desc: util::str_to_ptr(option.desc),
                            r#type: option.vtype.into(),
                        }))
                        .cast_const()
                    })
                    .chain(vec![std::ptr::null()])
                    .collect::<Vec<_>>(),
            )
            .as_ptr(),
            None => std::ptr::null(),
        };

        let plugin = proto::Plugin {
            id: util::str_to_ptr(self.id),
            name: util::str_to_ptr(self.name),
            desc: util::str_to_ptr(self.desc),
            variables,
            actions,
            config_options,
            fn_init,
            fn_update,
            fn_get_variable,
            fn_run_action,
            fn_get_enum,
            fn_get_config_value,
            fn_set_config_value,
        };

        Ok(Box::into_raw(Box::new(plugin)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Args, decorate_fn_get_variable, decorate_fn_init, decorate_fn_run_action,
        decorate_fn_update,
    };

    use super::*;

    #[test]
    fn test_build() {
        #![allow(clippy::unnecessary_wraps, clippy::trivially_copy_pass_by_ref)]

        fn init() -> Result<(), String> {
            Ok(())
        }

        fn update(_: &()) {}

        fn get_variable(_: &(), _: &str) -> Result<String, String> {
            Ok(String::new())
        }

        fn run_action(_: &(), _: &str, _: &Args) -> Result<(), String> {
            Ok(())
        }
        assert!(
            PluginBuilder::new("test_plugin", "Test Plugin", "Test Plugin")
                .init(decorate_fn_init!(init))
                .update(decorate_fn_update!(update))
                .get_variable(decorate_fn_get_variable!(get_variable))
                .run_action(decorate_fn_run_action!(run_action))
                .build()
                .is_ok()
        );
    }
}
