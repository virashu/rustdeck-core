use super::error::PluginLoadError;

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
