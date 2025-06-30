pub enum Type {
    Bool,
    Int,
    Float,
    String,
    Enum,
}

impl From<Type> for i32 {
    fn from(val: Type) -> Self {
        match val {
            Type::Bool => 0,
            Type::Int => 1,
            Type::Float => 2,
            Type::String => 3,
            Type::Enum => 4,
        }
    }
}

impl TryFrom<i32> for Type {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Bool,
            1 => Self::Int,
            2 => Self::Float,
            3 => Self::String,
            _ => panic!("Invalid type value"),
        })
    }
}

impl TryFrom<&str> for Type {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "bool" => Self::Bool,
            "int" => Self::Int,
            "float" => Self::Float,
            "string" => Self::String,
            "enum" => Self::Enum,
            _ => panic!("Invalid type value"),
        })
    }
}
