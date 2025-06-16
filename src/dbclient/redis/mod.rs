use redis::{FromRedisValue, RedisError};

use super::fetcher::{FetchResult, Fetcher, FetcherError};

pub struct RedisConfig {
    pub uri: String
}

pub struct RedisFetcher {
    pub config: RedisConfig
}

impl Fetcher for RedisFetcher {
    fn fetch(&mut self, request: &super::fetcher::FetchRequest) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        let client = redis::Client::open(self.config.uri.clone())?;
        let mut connection = client.get_connection()?;

        let cmd = request.query.first().ok_or(super::fetcher::FetcherError::InvalidQuery)?;

        let cmd = match cmd {
            super::query_builder::QueryElement::Operator(op) => Ok(op),
            super::query_builder::QueryElement::Parameter(_, _) => Err(FetcherError::InvalidQuery),
            super::query_builder::QueryElement::Select(_) => todo!(),
            super::query_builder::QueryElement::From(_) => todo!(),
            super::query_builder::QueryElement::Limit(_) => todo!(),
        }?;

        let mut res = &mut redis::cmd(cmd);
        for i in request.query.iter().skip(1) {
            match i {
                super::query_builder::QueryElement::Operator(op) => res = res.arg(op),
                super::query_builder::QueryElement::Parameter(name, value) => res = res.arg(&[name, value]),
                super::query_builder::QueryElement::Select(_) => todo!(),
                super::query_builder::QueryElement::From(_) => todo!(),
                super::query_builder::QueryElement::Limit(_) => todo!(),
            };
        }

        let res = res.query(&mut connection)?;
        Ok(res)
    }

    fn fetch_db_objects(&mut self) -> Result<FetchResult, FetcherError> {
        todo!()
    }
}

impl FromRedisValue for FetchResult {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Nil => Ok(FetchResult::none()),
            redis::Value::Int(x) => Ok(FetchResult::single(x)),
            redis::Value::BulkString(items) => Ok(FetchResult::multiple(items)),
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
                    res = FetchResult::merge(&res, &pre_res1);
                    res = FetchResult::merge(&res, &pre_res2);
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
