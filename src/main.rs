use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use config::Config;
use iocraft::{element, ElementExt};
use tokio::sync::Mutex;

use crate::ui4::app_state::{AppState, Page};
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
        Arc::new(core::server::ObjectsServer::new(config.connections));
    let app_state = Arc::new(Mutex::new(AppState {
        selected_connection: Some(core::proto::connections::Connection {
            id: "default".to_string(),
        }),
        selected_object: None,
        query_result_cmd: None,
        selected_page: Page::ConnectionsList,
    }));

    // element!(ui4::db_objects::DbObjects(
    //     objects_client: Some(db_objects_client),
    //     state: app_state
    // ))
    // .render_loop()
    // .await
    // .context("dbclient execution error")

    // element!(ui4::connections_list::ConnectionsList(
    //     connections_client: Some(connections_client),
    //     state: app_state,
    // ))
    // .render_loop()
    // .await
    // .context("dbclient execution error")

    element!(ui4::app_state::AppContainer(
        connections_client: Some(connections_client),
        objects_client: Some(db_objects_client),
        state: app_state,
    ))
    .render_loop()
    .await
    .context("app container execution error")
}
