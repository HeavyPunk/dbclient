use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Field {
    String(Option<String>),
    StringContainer(Option<Vec<String>>),
    Int8(Option<i8>),
    Int16(Option<i16>),
    Int32(Option<i32>),
    Int64(Option<i64>),
    Bool(Option<bool>),
    Time(Option<DateTime<Utc>>),
}

impl Field {
    pub fn as_type_str(&self) -> &'static str {
        match self {
            Field::String(_) => "string",
            Field::StringContainer(_) => "strings",
            Field::Int8(_) => "int8",
            Field::Int16(_) => "int16",
            Field::Int32(_) => "int32",
            Field::Int64(_) => "int64",
            Field::Bool(_) => "bool",
            Field::Time(_) => "time",
        }
    }

    pub fn as_sql_value(&self) -> String {
        match self {
            Field::String(str) => {
                match str {
                    Some(str) => format!("'{str}'"),
                    None => "".into()
                }
            },
            Field::StringContainer(items) => {
                match items {
                    Some(items) => {
                        let str = items.join("\n");
                        format!("'{str}'")
                    },
                    None => "".into()
                }
            }
            Field::Int8(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Int16(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Int32(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Int64(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Bool(b) => {
                match b {
                    Some(b) => format!("{b}"),
                    None => "".into()
                }
            },
            Field::Time(t) => {
                match t {
                    Some(t) => format!("{}", t.to_rfc2822()),
                    None => "".into()
                }
            },
        }
    }

    pub fn as_ui_value(&self) -> String {
        match self {
            Field::String(s) => {
                match s {
                    Some(s) => s.clone(),
                    None => "".into()
                }
            },
            Field::StringContainer(items) => {
                match items {
                    Some(items) => items.join("\n"),
                    None => "".into()
                }
            },
            Field::Int8(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Int16(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Int32(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Int64(i) => {
                match i {
                    Some(i) => format!("{i}"),
                    None => "".into()
                }
            },
            Field::Bool(b) => {
                match b {
                    Some(b) => format!("{b}"),
                    None => "".into()
                }
            },
            //TODO: make a valid time for sql
            Field::Time(t) => {
                match t {
                    Some(t) => format!("{}", t.to_rfc2822()),
                    None => "".into()
                }
            },
        }
    }
}

