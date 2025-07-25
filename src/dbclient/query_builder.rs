use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub enum QueryElement {
    RawQuery(String),
    ListAllItemsFrom(String),
    AddDatabaseObject(String, String, String)
}
