use crate::config::Connection;

pub mod model;
pub mod connections_list;
pub mod db_objects;

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    AppClose,
    ToQueryPage,
    ToConnectionsPage,
    ConnectionSelected(usize),
    SelectPrevConnection,
    SelectNextConnection,
    SelectNextDbObject,
    SelectPrevDbObject,
    FetchDbObjects,
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

