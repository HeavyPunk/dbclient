use std::collections::HashMap;

use dbclient::Field;

use crate::dbclient::fetcher::{FetchRequest, FetchResult};

pub mod connections_list;
pub mod db_objects;
pub mod editor_popup;
pub mod editor_simple_input;
pub mod model;
pub mod query_input;
pub mod query_result;

pub const APP_SEARCH_PATTERN: &str = "app-search-pattern";

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    AppClose,
    ToQueryPage(usize),
    ToConnectionsPage,
    FetchDbObjects,
    FetchDbObject(String),
    AddDbObject(String, String, String),
    AddRecordToDbObject(HashMap<String, Option<Field>>),
    ExecuteCustomQuery(String),
    ExecuteQuery(FetchRequest),
    EditorAccept,
    EditorPopupNext,
    EditorResult(EditorType, Id, HashMap<String, Option<Field>>),
    SearchPattern(String),
    ToQueryResultWidget,
    ToDbObjectsWidget,
    ActivateEditor(Id, EditorType),
    //TODO: optimize memory allocation by placing pointer
    DiactivateEditor,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd)]
pub enum AppEvent {}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    ConnectionsList,
    DbObjects,
    QueryLine,
    QueryResult,
}

pub enum Page {
    Connections,
    Query,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorType {
    Search,
    Query,
    AddDbObject,
    AddRecord,
}
