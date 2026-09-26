use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use config::Config;
use iocraft::{element, ElementExt};
use tokio::sync::Mutex;

use crate::ui4::app_state::{AppState, Page, QueryResultCmd};
mod cli;
mod config;
mod core;
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
        selected_page: Page::ConnectionsList,
        focus_widget: ui4::app_state::Widget::default(),
    }));

    let (query_result_cmd_sender, query_result_cmd_reader) =
        tokio::sync::mpsc::channel::<QueryResultCmd>(10);

    element!(ui4::app_state::AppContainer(
        connections_client: Some(connections_client),
        objects_client: Some(db_objects_client),
        queries_client: Some(queries_client),
        state: app_state,
        query_result_cmd_pipe: Some(query_result_cmd_sender),
        query_result_cmd_receiver: Some(query_result_cmd_reader),
    ))
    .render_loop()
    .await
    .context("app container execution error")
}
