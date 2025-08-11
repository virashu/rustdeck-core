use rustdeck_common::util::PtrToStrError;

use crate::plugins::util::TimeoutError;

#[derive(thiserror::Error, Debug)]
pub enum PluginLoadError {
    /// Error loading (not a .dll/.so)
    #[error("Not a library")]
    NotALibrary(libloading::Error),

    /// Error getting function
    #[error("Plugin missing exported `build` function")]
    SymbolError(libloading::Error),

    /// Other libloading errors
    #[error("Error loading library: {0}")]
    GenericLibError(libloading::Error),

    /// Plugin build function returned a null pointer
    #[error("Plugin build function returned a null pointer")]
    BuildError,

    /// Failed to read string
    #[error("Error reading from plugin: {0}")]
    ReadError(#[from] PtrToStrError),

    /// Error with id/name...
    #[error("Plugin format error: {0}")]
    FormatError(String),
}

fn win_error_to_err_code(err: &str) -> Option<i32> {
    err.split_once(',')
        .and_then(|x| x.0.strip_prefix("Os { code: "))
        .and_then(|x| x.parse().ok())
}

impl From<libloading::Error> for PluginLoadError {
    fn from(value: libloading::Error) -> Self {
        match value {
            libloading::Error::LoadLibraryExW { ref source }
                if win_error_to_err_code(&format!("{source:?}")) == Some(193) =>
            {
                Self::NotALibrary(value)
            }
            libloading::Error::GetProcAddress { ref source }
                if win_error_to_err_code(&format!("{source:?}")) == Some(127) =>
            {
                Self::SymbolError(value)
            }
            _ => Self::GenericLibError(value),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Plugin is not initialized")]
pub struct InitError();

#[derive(thiserror::Error, Debug)]
pub enum ActionError {
    #[error("Wrong action format: {0:?}")]
    InvalidFormat(String),

    #[error("Plugin {0:?} was not found")]
    PluginNotFound(String),

    #[error("Action {action:?} was not found for plugin {plugin:?}")]
    ActionNotFound { action: String, plugin: String },

    #[error("Arguments for action {0:?} did not pass validation")]
    InvalidArgs(String),

    #[error("Plugin returned an error: {0}")]
    PluginError(String),

    #[error(transparent)]
    InitError(#[from] InitError),

    #[error(transparent)]
    TimeoutError(#[from] TimeoutError),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum VariableError {
    #[error("Wrong variable format: {0:?}")]
    InvalidFormat(String),

    #[error("Plugin {0:?} was not found")]
    PluginNotFound(String),

    #[error("Variable {variable:?} was not found for plugin {plugin:?}")]
    VariableNotFound { variable: String, plugin: String },

    #[error("Plugin returned an error: {0}")]
    PluginError(String),

    #[error(transparent)]
    InitError(#[from] InitError),

    #[error(transparent)]
    TimeoutError(#[from] TimeoutError),
}
