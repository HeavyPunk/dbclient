use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Eq)]
pub enum ConnectionType {
    Redis,
    Postgres,
    MySql
}

impl fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let connection_type_str = match self {
            ConnectionType::Redis => "Redis",
            ConnectionType::Postgres => "Postgres",
            ConnectionType::MySql => "MySql",
        };
        write!(f, "{}", connection_type_str)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Eq)]
pub struct Connection {
    pub connection_type: ConnectionType,
    pub name: String,
    pub connection_string: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<Connection>
}
