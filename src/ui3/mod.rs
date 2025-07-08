use crate::dbclient::fetcher::{FetchRequest, FetchResult};

pub mod model;
pub mod connections_list;
pub mod db_objects;
pub mod query_result;
pub mod query_input;

pub const INPUT_POPUP_WIDGET_KIND: &str = "input-popup-widget-kind";
pub const APP_SEARCH_PATTERN: &str = "app-search-pattern";

#[derive(Debug)]
pub enum AppError {
    InternalError(&'static str)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    AppClose,
    ToQueryPage(usize),
    ToConnectionsPage,
    FetchDbObjects,
    FetchDbObject(String),
    ExecuteCustomQuery(String),
    ExecuteQuery(FetchRequest),
    EditorResult(WidgetKind, Vec<String>),
    SearchPattern(String),
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
#[repr(u8)]
pub enum WidgetKind {
    Query,
    Search
}

impl Into<u8> for WidgetKind {
    fn into(self) -> u8 {
        return self as u8;
    }
}

impl TryFrom<u8> for WidgetKind {
    type Error = AppError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::Search),
            _ => Err(AppError::InternalError("unexpected widget kind"))
        }
    }
}

