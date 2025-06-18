use std::{collections::HashMap, usize};

use crate::dbclient::{fetcher::{FetchRequest, FetchResult, Fetcher}, query_builder::QueryElement, redis::{RedisConfig, RedisFetcher}};


#[test]
fn test_int() {
    let mut redis = RedisFetcher {
        config: RedisConfig {
            uri: String::from("redis://127.0.0.1/")
        },
    };
    let result = redis.fetch(&FetchRequest{
        query: vec![
            QueryElement::RawQuery("GET test_int".to_string()),
        ],
        limit: 2,
    });

    let mut expected = HashMap::new();
    expected.insert("result".to_string(), vec!["49".to_string()]);
    assert_eq!(result, Ok(FetchResult{ table: Some(expected) }))
}

#[test]
fn test_string() {
    let mut redis = RedisFetcher {
        config: RedisConfig {
            uri: String::from("redis://127.0.0.1/")
        },
    };
    let result = redis.fetch(&FetchRequest{
        query: vec![
            QueryElement::ListAllItemsFrom("tags".to_string()),
        ],
        limit: usize::MAX,
    });

    let mut expected = HashMap::new();
    expected.insert("result".to_string(), vec!["nosql".to_string(), "redis".to_string(), "python".to_string()]);
    assert_eq!(result, Ok(FetchResult{ table: Some(expected) }))
}
