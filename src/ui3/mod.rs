use crate::dbclient::fetcher::{FetchRequest, FetchResult};

pub mod model;
pub mod connections_list;
pub mod db_objects;
pub mod query_result;
pub mod query_input;

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    AppClose,
    ToQueryPage(usize),
    ToConnectionsPage,
    FetchDbObjects,
    FetchDbObject(String),
    ExecuteCustomQuery(String),
    ExecuteQuery(FetchRequest),
    ToQueryResultWidget,
    ToDbObjectsWidget,
    ActivateEditor(WidgetKind),
    DiactivateEditor,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub enum AppEvent {
    ErrorInitialized,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    ConnectionsList,
    DbObjects,
    QueryLine,
    QueryResult,
}

pub enum Page {
    Connections,
    Query
}

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetKind {
    Query,
    Search
}

