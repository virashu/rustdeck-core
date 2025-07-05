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

impl Variable {
    pub fn new(id: impl Into<String>, desc: impl Into<String>, vtype: impl Into<Type>) -> Self {
        Self {
            id: id.into(),
            desc: desc.into(),
            vtype: vtype.into(),
        }
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

pub struct PluginBuilder {
    id: String,
    name: String,
    desc: String,

    variables: Option<Vec<Variable>>,
    actions: Option<Vec<Action>>,

    fn_init: Option<proto::FnInit>,
    fn_update: Option<proto::FnUpdate>,
    fn_get_variable: Option<proto::FnGetVariable>,
    fn_run_action: Option<proto::FnRunAction>,

    fn_get_enum: Option<proto::FnGetEnum>,
}

impl PluginBuilder {
    pub fn new(id: impl AsRef<str>, name: impl AsRef<str>, desc: impl AsRef<str>) -> Self {
        Self {
            id: id.as_ref().to_owned(),
            name: name.as_ref().to_owned(),
            desc: desc.as_ref().to_owned(),
            variables: None,
            actions: None,
            fn_init: None,
            fn_update: None,
            fn_get_variable: None,
            fn_run_action: None,
            fn_get_enum: None,
        }
    }

    #[must_use]
    pub fn init(mut self, f: proto::FnInit) -> Self {
        self.fn_init = Some(f);
        self
    }

    #[must_use]
    pub fn variable(mut self, var: Variable) -> Self {
        if let Some(vars) = &mut self.variables {
            vars.push(var);
        } else {
            self.variables = Some(vec![var]);
        }

        self
    }

    #[must_use]
    pub fn action(mut self, act: Action) -> Self {
        if let Some(acts) = &mut self.actions {
            acts.push(act);
        } else {
            self.actions = Some(vec![act]);
        }

        self
    }

    #[must_use]
    pub fn update(mut self, f: proto::FnUpdate) -> Self {
        self.fn_update = Some(f);
        self
    }

    #[must_use]
    pub fn get_variable(mut self, f: proto::FnGetVariable) -> Self {
        self.fn_get_variable = Some(f);
        self
    }

    #[must_use]
    pub fn run_action(mut self, f: proto::FnRunAction) -> Self {
        self.fn_run_action = Some(f);
        self
    }

    /// Builds the plugin.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the required functions are not set.
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

        #[allow(clippy::option_if_let_else)]
        let variables = match self.variables {
            Some(vars) => ManuallyDrop::new(
                vars.into_iter()
                    .map(|var| {
                        Box::into_raw(Box::new(proto::Variable {
                            id: util::str_to_ptr(var.id),
                            desc: util::str_to_ptr(var.desc),
                            r#type: var.vtype.into(),
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
        let actions = match self.actions {
            Some(acts) => ManuallyDrop::new(
                acts.into_iter()
                    .map(|act| {
                        Box::into_raw(Box::new(proto::Action {
                            id: util::str_to_ptr(act.id),
                            name: util::str_to_ptr(act.name),
                            desc: util::str_to_ptr(act.desc),
                            args: std::ptr::null(),
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
            fn_init,
            fn_update,
            fn_get_variable,
            fn_run_action,
            fn_get_enum,
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
