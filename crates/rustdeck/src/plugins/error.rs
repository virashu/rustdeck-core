use rustdeck_common::util::PtrToStrError;

#[derive(Debug)]
pub enum PluginLoadError {
    /// Error loading (not a .dll/.so)
    #[allow(dead_code)]
    NotALibrary(libloading::Error),
    /// Error getting function
    #[allow(dead_code)]
    SymbolError(libloading::Error),
    /// Plugin build funciton returned a null pointer
    BuildError,
    /// Other libloading errors
    GenericLibError(libloading::Error),
    /// Failed to read string
    ReadError(PtrToStrError),
    /// Error with id/name...
    FormatError(String),
}

impl From<libloading::Error> for PluginLoadError {
    fn from(value: libloading::Error) -> Self {
        match value {
            libloading::Error::LoadLibraryExW { ref source } => {
                let err_code: Result<i32, ()> = try {
                    format!("{source:?}")
                        .split_once(',')
                        .ok_or(())?
                        .0
                        .strip_prefix("Os { code: ")
                        .ok_or(())?
                        .parse()
                        .map_err(|_| ())?
                };

                match err_code {
                    Ok(193) => Self::NotALibrary(value),
                    _ => Self::GenericLibError(value),
                }
            }
            libloading::Error::GetProcAddress { ref source } => {
                let err_code: Result<i32, ()> = try {
                    format!("{source:?}")
                        .split_once(',')
                        .ok_or(())?
                        .0
                        .strip_prefix("Os { code: ")
                        .ok_or(())?
                        .parse()
                        .map_err(|_| ())?
                };

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
            Self::SymbolError(_) => write!(f, "Plugin missing exported `build` functiton"),
            Self::GenericLibError(e) => write!(f, "Error loading library: {e}"),
            Self::BuildError => write!(f, "Plugin build function returned a null pointer"),
            Self::ReadError(e) => write!(f, "Error reading from plugin: {e}"),
            Self::FormatError(e) => write!(f, "Plugin format error: {e}"),
        }
    }
}

impl std::error::Error for PluginLoadError {}
