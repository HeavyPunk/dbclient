use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Field {
    String(String),
    StringContainer(Vec<String>),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Bool(bool),
    Time(DateTime<Utc>),
}

impl Field {
    pub fn as_sql_value(&self) -> String {
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
            Field::Bool(b) => format!("{b}"),
            Field::Time(t) => format!("{}", t.to_rfc2822()),
        }
    }

    pub fn as_ui_value(&self) -> String {
        match self {
            Field::String(s) => s.clone(),
            Field::StringContainer(items) => items.join("\n"),
            Field::Int8(i) => format!("{i}"),
            Field::Int16(i) => format!("{i}"),
            Field::Int32(i) => format!("{i}"),
            Field::Int64(i) => format!("{i}"),
            Field::Bool(b) => format!("{b}"),
            //TODO: make a valid time for sql
            Field::Time(t) => format!("{}", t.to_rfc2822()),
        }
    }
}

