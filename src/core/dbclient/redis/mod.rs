use std::collections::HashMap;

use dbclient::Field;
use redis::{Cmd, Commands, Connection, FromRedisValue, RedisError};

use crate::core::dbclient::connector::ConnectorError;

pub struct RedisConfig {
    pub uri: String,
}

pub struct RedisFetcher {
    pub config: RedisConfig,
}

pub enum RedisType {
    String,
    List,
    Set,
    Zset,
    Hash,
    Stream,
    None,
}

impl<'a> TryFrom<&'a str> for RedisType {
    type Error = ConnectorError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "string" => Ok(RedisType::String),
            "list" => Ok(RedisType::List),
            "set" => Ok(RedisType::Set),
            "zset" => Ok(RedisType::Zset),
            "hash" => Ok(RedisType::Hash),
            "stream" => Ok(RedisType::Stream),
            "none" => Ok(RedisType::None),
            _ => Err(ConnectorError::InvalidRequest("unsupported type")),
        }
    }
}
