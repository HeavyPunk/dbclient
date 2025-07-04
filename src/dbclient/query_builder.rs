#[derive(Debug, PartialEq, Clone)]
pub enum QueryElement {
    RawQuery(String),
    ListAllItemsFrom(String),
}
