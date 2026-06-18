use std::collections::HashMap;

use dbclient::Field;

#[derive(Debug, PartialEq, Clone)]
pub enum QueryElement {
    RawQuery(String),
    ListAllItemsFrom(String),
    AddDatabaseObject(String, String, String),
    AddRecordToDbObject(String, HashMap<String, Option<Field>>),
}
