use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum QueryElement {
    RawQuery(String),
    ListAllItemsFrom(String),
    AddDatabaseObject(String, String, String),
    AddRecordToDbObject(String, HashMap<String, String>),
}
