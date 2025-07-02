use rustdeck_common::util::PtrToStrError;

#[derive(Debug)]
pub enum PluginLoadError {
    /// Error loading (not a .dll/.so)
    #[allow(dead_code)]
    NotALibrary(libloading::Error),
    /// Error getting function
    #[allow(dead_code)]
    SymbolError(libloading::Error),
    /// Plugin build function returned a null pointer
    BuildError,
    /// Initialization error
    InitError(String),
    /// Other libloading errors
    GenericLibError(libloading::Error),
    /// Failed to read string
    ReadError(PtrToStrError),
    /// Error with id/name...
    FormatError(String),
}

fn win_error_to_err_code(err: &str) -> Result<i32, ()> {
    err.split_once(',')
        .ok_or(())?
        .0
        .strip_prefix("Os { code: ")
        .ok_or(())?
        .parse()
        .map_err(|_| ())
}

impl From<libloading::Error> for PluginLoadError {
    fn from(value: libloading::Error) -> Self {
        match value {
            libloading::Error::LoadLibraryExW { ref source } => {
                let err_code = win_error_to_err_code(&format!("{source:?}"));

                match err_code {
                    Ok(193) => Self::NotALibrary(value),
                    _ => Self::GenericLibError(value),
                }
            }
            libloading::Error::GetProcAddress { ref source } => {
                let err_code = win_error_to_err_code(&format!("{source:?}"));

                match err_code {
                    Ok(127) => Self::SymbolError(value),
                    _ => Self::GenericLibError(value),
                }
            }
            _ => Self::GenericLibError(value),
        }
    }
}

impl From<PtrToStrError> for PluginLoadError {
    fn from(value: PtrToStrError) -> Self {
        Self::ReadError(value)
    }
}

impl std::fmt::Display for PluginLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotALibrary(_) => write!(f, "Not a library"),
            Self::SymbolError(_) => write!(f, "Plugin missing exported `build` function"),
            Self::GenericLibError(e) => write!(f, "Error loading library: {e}"),
            Self::BuildError => write!(f, "Plugin build function returned a null pointer"),
            Self::ReadError(e) => write!(f, "Error reading from plugin: {e}"),
            Self::FormatError(e) => write!(f, "Plugin format error: {e}"),
            Self::InitError(e) => write!(f, "Initialization error: {e}"),
        }
    }
}

impl std::error::Error for PluginLoadError {}

#[derive(thiserror::Error, Debug)]
pub enum ActionError {
    #[error("Wrong action format: '{0}'")]
    InvalidFormat(String),
    #[error("Plugin `{0}` was not found")]
    PluginNotFound(String),
    #[error("Action `{action}` was not found for plugin `{plugin}`")]
    ActionNotFound { action: String, plugin: String },
    #[error("Arguments for action `{0}` did not pass validation")]
    InvalidArgs(String),
}
