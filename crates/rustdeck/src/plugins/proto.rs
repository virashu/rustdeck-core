use rustdeck_common::{
    proto::{
        Action as FFIAction, ActionArg as FFIActionArg, ConfigOption as FFIConfigOption,
        Variable as FFIVariable,
    },
    util,
};

use crate::plugins::error::PluginLoadError;

use super::datatype::PluginDataType;

/// Args are positional
#[derive(Clone)]
pub struct ActionArg {
    pub id: String,
    pub name: String,
    pub description: String,
    pub r#type: PluginDataType,
}

impl ActionArg {
    pub fn from_ffi_ref(value: &FFIActionArg) -> Result<Self, PluginLoadError> {
        Ok(Self {
            id: unsafe { util::try_ptr_to_str(value.id) }?.to_owned(),
            name: unsafe { util::try_ptr_to_str(value.name) }?.to_owned(),
            description: unsafe { util::try_ptr_to_str(value.desc) }?.to_owned(),
            r#type: value.r#type.try_into()?,
        })
    }
}

#[derive(Clone)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<ActionArg>,
}

impl Action {
    pub fn from_ffi_ref(value: &FFIAction) -> Result<Self, PluginLoadError> {
        Ok(Self {
            id: unsafe { util::try_ptr_to_str(value.id) }?.to_owned(),
            name: unsafe { util::try_ptr_to_str(value.name) }?.to_owned(),
            description: unsafe { util::try_ptr_to_str(value.desc) }?.to_owned(),
            args: if value.args.is_null() {
                Vec::new()
            } else {
                (0..isize::MAX)
                    .map_while(|offset| unsafe {
                        value.args.offset(offset).as_ref().unwrap().as_ref()
                    })
                    .map(ActionArg::from_ffi_ref)
                    .collect::<Result<Vec<ActionArg>, PluginLoadError>>()?
            },
        })
    }
}

#[derive(Clone)]
pub struct Variable {
    pub id: String,
    pub description: String,
    pub r#type: PluginDataType,
}

impl Variable {
    pub fn from_ffi_ref(value: &FFIVariable) -> Result<Self, PluginLoadError> {
        Ok(Self {
            id: unsafe { util::try_ptr_to_str(value.id) }?.to_owned(),
            description: unsafe { util::try_ptr_to_str(value.desc) }?.to_owned(),
            r#type: value.r#type.try_into()?,
        })
    }
}

#[derive(Clone)]
pub struct ConfigOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub r#type: PluginDataType,
}

impl ConfigOption {
    pub fn from_ffi_ref(value: &FFIConfigOption) -> Result<Self, PluginLoadError> {
        Ok(Self {
            id: unsafe { util::try_ptr_to_str(value.id) }?.to_owned(),
            name: unsafe { util::try_ptr_to_str(value.name) }?.to_owned(),
            description: unsafe { util::try_ptr_to_str(value.desc) }?.to_owned(),
            r#type: value.r#type.try_into()?,
        })
    }
}
