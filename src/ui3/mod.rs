use std::{collections::HashMap, path::Path};

use crate::dbclient::fetcher::{FetchRequest, FetchResult};

pub mod model;
pub mod connections_list;
pub mod db_objects;
pub mod query_result;
pub mod query_input;
pub mod editor_popup;

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
    AddDbObject(String, String, String),
    ExecuteCustomQuery(String),
    ExecuteQuery(FetchRequest),
    EditorAccept,
    EditorPopupNext,
    EditorResult(EditorType, HashMap<&'static str, Vec<String>>),
    SearchPattern(String),
    ToQueryResultWidget,
    ToDbObjectsWidget,
    ActivateEditor(EditorType),
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
pub enum EditorType {
    Search,
    Query,
    AddDbObject,
}

