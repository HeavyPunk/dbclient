use crate::dbclient::{fetcher::{FetchRequest, FetchResult, Fetcher, Row}, query_builder::QueryElement, redis::{RedisConfig, RedisFetcher}};


#[test]
fn test_int() {
    let mut redis = RedisFetcher {
        config: RedisConfig {
            uri: String::from("redis://127.0.0.1/")
        },
    };
    let result = redis.fetch(&FetchRequest{
        query: vec![
            QueryElement::Operator(String::from("GET")),
            QueryElement::Operator(String::from("test_int"))
        ],
        limit: 2,
    });

    assert_eq!(result, Ok(FetchResult{
        header: Some(Row { columns: vec![String::from("result")] }),
        rows: Some(vec![Row { columns: vec![String::from("49")] }])
    }))
}
