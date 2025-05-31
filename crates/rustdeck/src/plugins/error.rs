#[derive(Debug)]
pub enum PluginLoadError {
    /// Error loading (not a .dll/.so)
    NotALibrary(libloading::Error),
    /// Other libloading errors
    GenericLibError(libloading::Error),
    /// Cannot get needed function (`get_name`, etc.)
    SymbolError(std::str::Utf8Error),
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
            _ => Self::GenericLibError(value),
        }
    }
}

impl From<std::str::Utf8Error> for PluginLoadError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::SymbolError(value)
    }
}

impl std::fmt::Display for PluginLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotALibrary(e) => write!(f, "Not a library: {e}"),
            Self::GenericLibError(e) => write!(f, "Error loading library: {e}"),
            Self::SymbolError(e) => write!(f, "Symbol error: {e}"),
            Self::FormatError(e) => write!(f, "Plugin format error: {e}"),
        }
    }
}

impl std::error::Error for PluginLoadError {}
