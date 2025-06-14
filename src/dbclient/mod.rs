//#[cfg(feature = "redis")]
pub mod redis;

pub mod query_builder;

pub(crate) mod fetcher {
    use super::query_builder::QueryElement;


    #[derive(Debug, PartialEq)]
    pub struct FetchResult {
        pub header: Option<Row>,
        pub rows: Option<Vec<Row>>
    }

    #[derive(Debug, PartialEq)]
    pub struct Row {
        pub columns: Vec<String>
    }

    #[derive(Debug)]
    pub struct FetchRequest {
        pub query: Vec<QueryElement>,
        pub limit: usize,
    }

    #[derive(Debug, PartialEq)]
    pub enum FetcherError {
        InvalidQuery,
        RedisError(redis::RedisError),
    }

    impl From<redis::RedisError> for FetcherError {
        fn from(err: redis::RedisError) -> Self {
            FetcherError::RedisError(err)
        }
    }

    pub trait Fetcher {
        fn fetch(&mut self, request: &FetchRequest) -> Result<FetchResult, FetcherError>;
    }

    impl FetchResult {
        pub fn none() -> FetchResult {
            FetchResult { header: None, rows: None }
        }

        pub fn single<T>(item: &T) -> FetchResult where T: ToString {
            FetchResult {
                header: Some(
                    Row {
                        columns: vec![String::from("result")]
                    }
                ),
                rows: Some(vec![Row {
                    columns: vec![item.to_string()]
                }])
            }
        }

        pub fn multiple<T>(items: &Vec<T>) -> FetchResult where T: ToString {
            FetchResult {
                header: Some(Row {
                    columns: vec![String::from("result")]
                }),
                rows: Some(items.iter().map(|item| Row {columns: vec![item.to_string()]}).collect())
            }
        }

        pub fn merge(result1: &FetchResult, result2: &FetchResult) -> FetchResult {
            todo!()
        }
    }
}

