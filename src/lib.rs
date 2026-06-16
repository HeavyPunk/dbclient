#[derive(Debug, PartialEq, Clone)]
pub enum Field {
    String(String),
    StringContainer(Vec<String>),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
}

pub type SQLRepresentation = String;

impl Into<SQLRepresentation> for Field {
    fn into(self) -> SQLRepresentation {
        match self {
            Field::String(str) => format!("'{str}'"),
            Field::StringContainer(items) => {
                let str = items.join("\n");
                format!("'{str}'")
            }
            Field::Int8(i) => format!("{i}"),
            Field::Int16(i) => format!("{i}"),
            Field::Int32(i) => format!("{i}"),
            Field::Int64(i) => format!("{i}"),
        }
    }
}
