#[derive(Debug)]
pub enum QueryElement {
    RawQuery(String),
    ListAllItemsFrom(String),
}
