//#[cfg(feature = "redis")]
pub mod redis;
pub mod dummy;

pub mod query_builder;

pub(crate) mod fetcher {
    use std::collections::HashMap;

    use super::query_builder::QueryElement;


    #[derive(Debug, PartialEq)]
    pub struct FetchResult {
        pub table: Option<HashMap<String, Vec<String>>>,
    }

    impl FetchResult {
        pub fn get_table_height(&self) -> usize {
            match &self.table {
                Some(table) => table.values().map(|v| v.len()).max().unwrap_or(0),
                None => 0,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
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
        fn fetch_db_objects(&mut self) -> Result<FetchResult, FetcherError>;
        fn fetch(&mut self, request: &FetchRequest) -> Result<FetchResult, FetcherError>;
    }

    impl FetchResult {
        pub fn none() -> FetchResult {
            FetchResult { table: None }
        }

        pub fn single<T>(item: &T) -> FetchResult where T: ToString {
            let mut table = HashMap::new();
            table.insert("result".to_string(), vec![item.to_string()]);

            FetchResult {
                table: Some(table),
            }
        }

        pub fn multiple<T>(items: &Vec<T>) -> FetchResult where T: ToString {
            let mut table = HashMap::new();
            table.insert("result".to_string(), items.iter().map(|item| item.to_string()).collect());
            FetchResult { table: Some(table) }
        }

        pub fn merge(result1: &FetchResult, result2: &FetchResult) -> FetchResult {
            let table = match (result1.table.clone(), result2.table.clone()) {
                (None, None) => None,
                (None, Some(t)) => Some(t),
                (Some(t), None) => Some(t),
                (Some(t1), Some(t2)) => {
                    let mut merged_table = t1.clone();
                    for (key, value) in t2 {
                        merged_table.entry(key).or_insert_with(Vec::new).extend(value);
                    }
                    Some(merged_table)
                },
            };
            FetchResult { table }
        }
    }
}

