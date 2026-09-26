use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dbclient::Field;

use crate::core::dbclient::query_builder::QueryElement;

pub mod connector_impl;

pub struct PostgresConfig {
    pub uri: String,
}

pub struct PostgresFetcher {
    pub config: PostgresConfig,
}
