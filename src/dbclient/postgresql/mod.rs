use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dbclient::Field;
use postgres::{Column, NoTls, Row};

use crate::dbclient::{
    fetcher::{FetchResult, Fetcher, FetcherError},
    query_builder::QueryElement,
};

pub struct PostgresConfig {
    pub uri: String,
}

pub struct PostgresFetcher {
    pub config: PostgresConfig,
}

impl From<postgres::Error> for super::fetcher::FetcherError {
    fn from(value: postgres::Error) -> Self {
        Self::PostgresError(value)
    }
}

impl Fetcher for PostgresFetcher {
    fn fetch_db_objects(
        &mut self,
    ) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        let mut client = postgres::Client::connect(&self.config.uri, NoTls)?;
        let query = "
SELECT
    table_schema,
    table_name
FROM information_schema.tables
WHERE table_schema NOT IN (
    'pg_catalog',
    'information_schema'
)
ORDER BY table_schema, table_name;
        ";
        let query_res = client.query(query, &[])?;
        let mut table: HashMap<String, Vec<Field>> = HashMap::new();
        for row in query_res {
            let table_schema: String = row.get(0);
            let table_name: String = row.get(1);
            if let Some(tables) = table.get_mut(&table_schema) {
                tables.push(Field::String(Some(table_name)));
            } else {
                table.insert(table_schema, vec![Field::String(Some(table_name))]);
            }
        }
        Ok(super::fetcher::FetchResult::new((vec![], table)))
    }

    fn fetch(
        &mut self,
        request: &super::fetcher::FetchRequest,
    ) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        let mut client = postgres::Client::connect(&self.config.uri, NoTls)?;

        match request.query.first() {
            Some(query) => match query {
                QueryElement::RawQuery(query) => {
                    let rows = client.query(query, &[])?;
                    FetchResult::from_postgres_value(rows)
                }
                QueryElement::ListAllItemsFrom(index) => {
                    let query = format!("SELECT * FROM {}", index);
                    let rows = client.query(&query, &[])?;
                    FetchResult::from_postgres_value(rows)
                }
                QueryElement::AddDatabaseObject(_, _, _) => {
                    //TODO: need to extend database object context to create a table
                    unimplemented!()
                }
                QueryElement::AddRecordToDbObject(db_object, fields) => {
                    let mut keys_vec = vec![];
                    let mut values_vec: Vec<String> = vec![];
                    for pair in fields {
                        let (key, val) = (pair.0, pair.1);
                        match val {
                            Field::String(None)
                            | Field::StringContainer(None)
                            | Field::Int8(None)
                            | Field::Int16(None)
                            | Field::Int32(None)
                            | Field::Int64(None)
                            | Field::Bool(None)
                            | Field::Time(None) => continue,
                            _ => (),
                        };
                        keys_vec.push(key.clone());
                        values_vec.push(val.clone().as_sql_value());
                    }
                    let keys = keys_vec.join(",");
                    let values = values_vec.join(",");

                    let query = format!("INSERT INTO {db_object} ({keys}) VALUES ({values})");
                    client.execute(&query, &[])?;
                    Ok(FetchResult::none())
                }
                QueryElement::UpdateRecord(db_object, fields, table_indexes) => {
                    let mut updates = vec![];
                    let mut indexes = vec![];
                    for pair in fields {
                        let (key, val) = (pair.0, pair.1);
                        match val {
                            Field::String(None)
                            | Field::StringContainer(None)
                            | Field::Int8(None)
                            | Field::Int16(None)
                            | Field::Int32(None)
                            | Field::Int64(None)
                            | Field::Bool(None)
                            | Field::Time(None) => continue,
                            _ => (),
                        };
                        updates.push(format!("{}={}", key, val.as_sql_value()));
                    }
                    for pair in table_indexes {
                        let (key, val) = (pair.0, pair.1);
                        indexes.push(format!("{}={}", key, val.as_sql_value()));
                    }
                    let updates = updates.join(",");
                    let indexes = indexes.join(" AND ");

                    let query = format!("UPDATE {db_object} SET {updates} WHERE {indexes}");
                    client.execute(&query, &[])?;
                    Ok(FetchResult::none())
                }
            },
            None => Err(FetcherError::InvalidQuery),
        }
    }
}

pub enum PostgresType {
    Table,
}

impl<'a> TryFrom<&'a str> for PostgresType {
    type Error = FetcherError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "table" => Ok(PostgresType::Table),
            _ => Err(FetcherError::InvalidQuery),
        }
    }
}

impl FetchResult {
    fn from_postgres_value(rows: Vec<Row>) -> Result<FetchResult, FetcherError> {
        let mut table: HashMap<String, Vec<Field>> = HashMap::new();
        for row in rows {
            let columns = row.columns();
            for column in columns {
                let name = column.name();
                let value = FetchResult::map_column_to_value(column, &row)?;
                if let Some(table_column) = table.get_mut(name) {
                    table_column.push(value);
                } else {
                    table.insert(name.to_string(), vec![value]);
                }
            }
        }
        let keys: Vec<String> = table.keys().cloned().collect();
        Ok(FetchResult::new((keys, table)))
    }

    fn map_column_to_value(column: &Column, row: &Row) -> Result<Field, FetcherError> {
        let name = column.name();
        let column_type = column.type_();
        let column_name = column_type.name();
        match column_name {
            "bool" => {
                let value: bool = row.try_get(name)?;
                Ok(Field::Bool(Some(value)))
            }
            "int2" => {
                let value: i16 = row.try_get(name)?;
                Ok(Field::Int16(Some(value)))
            }
            "int4" => {
                let value: i32 = row.try_get(name)?;
                Ok(Field::Int32(Some(value)))
            }
            "int8" => {
                let value: i64 = row.try_get(name)?;
                Ok(Field::Int64(Some(value)))
            }
            "varchar" => {
                let value: String = row.try_get(name)?;
                Ok(Field::String(Some(value)))
            }
            "timestamptz" => {
                let value: std::time::SystemTime = row.try_get(name)?;
                let datetime: DateTime<Utc> = value.into();
                Ok(Field::Time(Some(datetime)))
            }
            _ => Err(FetcherError::StringMappingError(format!(
                "[postgresql] failed to map {column_name}"
            ))),
        }
    }
}
