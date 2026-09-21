use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use config::Config;
use iocraft::{element, ElementExt};
use tokio::sync::Mutex;

use crate::{
    core::proto,
    ui4::app_state::{AppState, Page},
};
mod cli;
mod config;
mod core;
mod dbclient;
mod ui3;
mod ui4;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let config_content =
        std::fs::read_to_string(&args.config_path).expect("Failed to read config file");

    let config: Config = toml::from_str(&config_content).expect("Failed to parse config file");

    let connections_client: Arc<dyn core::proto::connections::ConnectionsService + Send + Sync> =
        Arc::new(core::server::ConnectionsServer::new(
            config.connections.clone(),
        ));

    let db_objects_client: Arc<dyn core::proto::objects::ObjectsService + Send + Sync> =
        Arc::new(core::server::ObjectsServer::new(&config.connections));

    let queries_client: Arc<dyn core::proto::queries::QueriesService + Send + Sync> =
        Arc::new(core::server::QueriesServer::new(&config.connections));
    let app_state = Arc::new(Mutex::new(AppState {
        selected_connection: Some(core::proto::connections::Connection {
            id: "default".to_string(),
        }),
        selected_object: None,
        query_result_cmd: None,
        selected_page: Page::ConnectionsList,
        focus_widget: ui4::app_state::Widget::default(),
    }));

    element!(ui4::app_state::AppContainer(
        connections_client: Some(connections_client),
        objects_client: Some(db_objects_client),
        queries_client: Some(queries_client),
        state: app_state,
    ))
    .render_loop()
    .await
    .context("app container execution error")

    // element!(ui4::forms::add_record_to_object::AddRecordToObjectForm(
    //     object: proto::common::DbObject {
    //         specification: Some(proto::common::db_object::Specification::Postgres(proto::common::PostgresObject {
    //             object: Some(proto::common::postgres_object::Object::Table(proto::common::PostgresTable {
    //                 descriptor: None,
    //                 columns: vec![
    //                     proto::common::PostgresTableColumn {
    //                         name: "field-name-1".to_string(),
    //                         field: Some(proto::common::DbField {
    //                             field: Some(proto::common::db_field::Field::I8(5))
    //                         })
    //                     },
    //                     proto::common::PostgresTableColumn {
    //                         name: "field-name-2".to_string(),
    //                         field: Some(proto::common::DbField {
    //                             field: Some(proto::common::db_field::Field::Str("hahaha".to_string()))
    //                         })
    //                     },
    //                 ],
    //                 constrains: vec![],
    //                 indexes: vec![]
    //             }))
    //         }))
    //     }
    // ))
    // .render_loop()
    // .await
    // .context("app container execution error")
}
