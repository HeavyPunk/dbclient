use std::collections::HashMap;

use redis::{Cmd, Commands, Connection, ConnectionLike as _, FromRedisValue, RedisError, Value};

use super::{fetcher::{FetchResult, Fetcher, FetcherError}, query_builder::QueryElement};

pub struct RedisConfig {
    pub uri: String
}

pub struct RedisFetcher {
    pub config: RedisConfig
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

impl Fetcher for RedisFetcher {
    fn fetch(&mut self, request: &super::fetcher::FetchRequest) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        let client = redis::Client::open(self.config.uri.clone())?;
        let mut connection = client.get_connection()?;

        match request.query.first() {
            Some(query) => match query {
                QueryElement::RawQuery(query) => {
                    let mut cmd = Cmd::new();
                    let args: Vec<&str> = query.split(' ').collect();
                    cmd.arg(args);

                    let res = cmd.query(&mut connection)?;

                    Ok(FetchResult::from_redis_value(&res)?)
                },
                QueryElement::ListAllItemsFrom(index) => {
                    let index_type = get_index_type(index, &mut connection)?;
                    let res = match index_type {
                        RedisType::String => {
                            let res: String = connection.get(index)?;
                            FetchResult::single(&res)
                        },
                        RedisType::List => {
                            let res: Vec<String> = connection.lrange(index, 0, -1)?;
                            FetchResult::multiple(&res)
                        },
                        RedisType::Set => {
                            let res: Vec<String> = connection.smembers(index)?;
                            FetchResult::multiple(&res)
                        },
                        RedisType::Zset => {
                            let res: Vec<String> = connection.zrange(index, 0, -1)?;
                            FetchResult::multiple(&res)
                        },
                        RedisType::Hash => {
                            let res: HashMap<String, String> = connection.hgetall(index)?;
                            FetchResult::key_value(res)
                        },
                        RedisType::Stream => {
                            FetchResult::none()
                        },
                        RedisType::None => {
                            FetchResult::none()
                        },
                    };
                    
                    // Ok(FetchResult::from_redis_value(&res)?)
                    Ok(res)
                },
            },
            None => Err(FetcherError::InvalidQuery),
        }
    }

    fn fetch_db_objects(&mut self) -> Result<FetchResult, FetcherError> {
        let client = redis::Client::open(self.config.uri.clone())?;
        let mut connection = client.get_connection()?;

        let mut cursor = 0;
        let mut res = FetchResult::none();

        loop {
            let mut cmd = redis::cmd("SCAN");
            let cmd = cmd.arg(cursor).arg("MATCH").arg("*");
            let scan_res: (u64, Vec<String>) = cmd.query(&mut connection)?;

            cursor = scan_res.0;
            let keys = FetchResult::from_redis_value(&redis::Value::Array(
                scan_res.1.into_iter().map(redis::Value::SimpleString).collect(),
            ))?;
            res = FetchResult::merge(&res, &keys);

            if cursor == 0 {
                break;
            }
        }

        Ok(res)
    }
}

fn get_index_type(index: &String, mut connection: &mut Connection) -> Result<RedisType, FetcherError> {
    let mut type_cmd = redis::cmd("TYPE");
    let type_cmd = type_cmd.arg(index);
    let type_res = type_cmd.query::<FetchResult>(&mut connection)?;
    match type_res.table {
        Some(table) => {
            match table.1.iter().last() {
                Some((_, column)) => match column.first() {
                    Some(val) => match val.as_str() {
                        "string" => Ok(RedisType::String),
                        "list" => Ok(RedisType::List),
                        "set" => Ok(RedisType::Set),
                        "zset" => Ok(RedisType::Zset),
                        "hash" => Ok(RedisType::Hash),
                        "stream" => Ok(RedisType::Stream),
                        "none" => Ok(RedisType::None),
                        _ => Err(FetcherError::InvalidQuery)
                    },
                    None => Err(FetcherError::InvalidQuery),
                },
                None => Err(FetcherError::InvalidQuery),
            }
        },
        None => Err(FetcherError::InvalidQuery),
    }
}

impl FromRedisValue for FetchResult {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Nil => Ok(FetchResult::none()),
            redis::Value::Int(x) => Ok(FetchResult::single(x)),
            redis::Value::BulkString(items) => {
                let string_result = String::from_utf8(items.clone()).map_err(|_| RedisError::from((redis::ErrorKind::TypeError, "Invalid UTF-8 in BulkString")))?;
                Ok(FetchResult::single(&string_result))
            },
            redis::Value::Array(values) => {
                let mut res = FetchResult::none();
                for value in values {
                    let pre_res = FetchResult::from_redis_value(value)?;
                    res = FetchResult::merge(&res, &pre_res);
                }
                Ok(res)
            },
            redis::Value::SimpleString(item) => Ok(FetchResult::single(item)),
            redis::Value::Okay => Ok(FetchResult::none()),
            redis::Value::Map(items) => {
                let mut res = FetchResult::none();
                for item in items {
                    let pre_res1 = FetchResult::from_redis_value(&item.0)?;
                    let pre_res2 = FetchResult::from_redis_value(&item.1)?;
                    res = FetchResult::join(&res, &pre_res1);
                    res = FetchResult::join(&res, &pre_res2);
                }
                Ok(res)
            },
            redis::Value::Attribute { data: _, attributes: _ } => unimplemented!(),
            redis::Value::Set(values) => {
                let mut res = FetchResult::none();
                for value in values {
                    let pre_res = FetchResult::from_redis_value(value)?;
                    res = FetchResult::merge(&res, &pre_res);
                }
                Ok(res)
            },
            redis::Value::Double(x) => Ok(FetchResult::single(x)),
            redis::Value::Boolean(x) => Ok(FetchResult::single(x)),
            redis::Value::VerbatimString { format: _, text } => Ok(FetchResult::single(text)),
            redis::Value::BigNumber(big_int) => Ok(FetchResult::single(big_int)),
            redis::Value::Push { kind: _, data: _ } => unimplemented!(),
            redis::Value::ServerError(server_error) => Err(RedisError::from(server_error.clone())),
        }
    }
}

#[cfg(test)]
mod integration_tests;
