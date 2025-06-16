#[derive(Debug)]
pub enum QueryElement {
    Operator(String),
    Parameter(String, String),
    Select(String),
    From(String),
    Limit(String),
}
