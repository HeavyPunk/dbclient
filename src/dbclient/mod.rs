//#[cfg(feature = "redis")]
pub mod dummy;
pub mod postgresql;
pub mod redis;

pub mod query_builder;

pub(crate) mod fetcher {
    use std::collections::HashMap;

    use dbclient::Field;

use super::query_builder::QueryElement;

    type IndexColumn = String;

    #[derive(Debug, PartialEq, Clone)]
    pub enum FetchResult {
        None,
        Table(Option<(Vec<IndexColumn>, HashMap<String, Vec<Field>>)>),
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Row {
        pub columns: Vec<String>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct FetchRequest {
        pub query: Vec<QueryElement>,
        pub limit: usize,
    }

    #[derive(Debug)]
    pub enum FetcherError {
        InvalidQuery,
        MappingError(String),
        RedisError(redis::RedisError),
        PostgresError(postgres::Error),
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
            FetchResult::None
        }

        pub fn new(items: (Vec<IndexColumn>, HashMap<String, Vec<Field>>)) -> FetchResult {
            FetchResult::Table(Some(items))
        }

        pub fn single<T>(item: &T) -> FetchResult
        where
            T: ToString,
        {
            let mut table = HashMap::new();
            let index_column = "result".to_string();
            table.insert(index_column.clone(), vec![Field::String(Some(item.to_string()))]);

            FetchResult::Table(Some((vec![index_column], table)))
        }

        pub fn multiple<T>(items: &Vec<T>) -> FetchResult
        where
            T: ToString,
        {
            let mut table = HashMap::new();
            let index_column = "result".to_string();
            table.insert(
                index_column.clone(),
                items.iter().map(|item| Field::String(Some(item.to_string()))).collect(),
            );
            FetchResult::Table(Some((vec![index_column], table)))
        }

        pub fn key_value(items: HashMap<Field, Field>) -> FetchResult {
            let keys: Vec<_> = items.keys().cloned().collect();
            let values: Vec<_> = items.values().cloned().collect();
            let mut table = HashMap::new();
            let index_column = "keys".to_string();
            table.insert(index_column.clone(), keys);
            table.insert("values".to_string(), values);
            FetchResult::Table(Some((vec![index_column], table)))
        }

        pub fn merge(result1: &FetchResult, result2: &FetchResult) -> FetchResult {
            let table = match (result1.clone(), result2.clone()) {
                (FetchResult::None, _) | (_, FetchResult::None) => None,
                (FetchResult::Table(table1), FetchResult::Table(table2)) => {
                    match (table1, table2) {
                        (None, None) => None,
                        (None, Some(t)) => Some(t),
                        (Some(t), None) => Some(t),
                        (Some(t1), Some(t2)) => {
                            let mut merged_table = t1.clone();
                            for (key, value) in t2.1 {
                                merged_table
                                    .1
                                    .entry(key)
                                    .or_insert_with(Vec::new)
                                    .extend(value);
                            }
                            Some(merged_table)
                        }
                    }
                }
            };
            FetchResult::Table(table)
        }

        pub fn join(result1: &FetchResult, result2: &FetchResult) -> FetchResult {
            let table = match (&result1, &result2) {
                (FetchResult::None, _) | (_, FetchResult::None) => None,
                (FetchResult::Table(table1), FetchResult::Table(table2)) => {
                    match (table1, table2) {
                        (None, None) => None,
                        (None, Some(t)) => Some((t.0.clone(), t.1.clone())),
                        (Some(t), None) => Some((t.0.clone(), t.1.clone())),
                        (Some(t1), Some(t2)) => {
                            let mut merged_table: HashMap<String, Vec<Field>> = HashMap::new();
                            let mut index_keys = t1.0.clone();
                            index_keys.extend(t2.0.clone());
                            for (key, value) in &t1.1 {
                                merged_table
                                    .insert(format!("{}_1", key).to_string(), value.to_vec());
                            }
                            for (key, value) in &t2.1 {
                                merged_table
                                    .insert(format!("{}_2", key).to_string(), value.to_vec());
                            }
                            Some((index_keys, merged_table))
                        }
                    }
                }
            };
            FetchResult::Table(table)
        }
    }
}
