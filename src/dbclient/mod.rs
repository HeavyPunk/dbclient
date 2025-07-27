//#[cfg(feature = "redis")]
pub mod redis;
pub mod dummy;

pub mod query_builder;

pub(crate) mod fetcher {
    use std::collections::HashMap;

    use super::query_builder::QueryElement;

    type IndexColumn = String;

    #[derive(Debug, PartialEq, Clone)]
    pub struct FetchResult {
        pub table: Option<(Vec<IndexColumn>, HashMap<String, Vec<String>>)>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Row {
        pub columns: Vec<String>
    }

    #[derive(Debug, PartialEq, Clone)]
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
            let index_column = "result".to_string();
            table.insert(index_column.clone(), vec![item.to_string()]);

            FetchResult {
                table: Some((vec![index_column], table)),
            }
        }

        pub fn multiple<T>(items: &Vec<T>) -> FetchResult where T: ToString {
            let mut table = HashMap::new();
            let index_column = "result".to_string();
            table.insert(index_column.clone(), items.iter().map(|item| item.to_string()).collect());
            FetchResult { table: Some((vec![index_column], table)) }
        }

        pub fn key_value(items: HashMap<String, String>) -> FetchResult {
            let keys: Vec<String> = items.keys().cloned().collect();
            let values: Vec<String> = items.values().cloned().collect();
            let mut table = HashMap::new();
            let index_column = "keys".to_string();
            table.insert(index_column.clone(), keys);
            table.insert("values".to_string(), values);
            FetchResult { table: Some((vec![index_column], table)) }
        }

        pub fn merge(result1: &FetchResult, result2: &FetchResult) -> FetchResult {
            let table = match (result1.table.clone(), result2.table.clone()) {
                (None, None) => None,
                (None, Some(t)) => Some(t),
                (Some(t), None) => Some(t),
                (Some(t1), Some(t2)) => {
                    let mut merged_table = t1.clone();
                    for (key, value) in t2.1 {
                        merged_table.1.entry(key).or_insert_with(Vec::new).extend(value);
                    }
                    Some(merged_table)
                },
            };
            FetchResult { table }
        }

        pub fn join(result1: &FetchResult, result2: &FetchResult) -> FetchResult {
            let table = match (&result1.table, &result2.table) {
                (None, None) => None,
                (None, Some(t)) => Some((t.0.clone(), t.1.clone())),
                (Some(t), None) => Some((t.0.clone(), t.1.clone())),
                (Some(t1), Some(t2)) => {
                    let mut merged_table: HashMap<String, Vec<String>> = HashMap::new();
                    let mut index_keys = t1.0.clone();
                    index_keys.extend(t2.0.clone());
                    for (key, value) in &t1.1 {
                        merged_table.insert(format!("{}_1", key).to_string(), value.to_vec());
                    }
                    for (key, value) in &t2.1 {
                        merged_table.insert(format!("{}_2", key).to_string(), value.to_vec());
                    }
                    Some((index_keys, merged_table))
                },
            };
            FetchResult { table: table }
        }
    }
}

