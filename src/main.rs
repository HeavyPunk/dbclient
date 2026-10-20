use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use config::Config;
use dbclient::virtual_logger::{Notification, VirtualLogger};
use flexi_logger::{FileSpec, Logger, WriteMode};
use iocraft::{element, ElementExt};
use tokio::sync::Mutex;

use crate::ui4::app_state::{AppState, Page, QueryResultCmd};
mod cli;
mod config;
mod core;
mod ui4;

const DBCLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    let (notification_sender, notification_receiver) =
        tokio::sync::mpsc::channel::<Notification>(10);

    init_logger(true, Some(notification_sender))?;

    let args = Cli::parse();

    let config_content =
        std::fs::read_to_string(&args.config_path).expect("Failed to read config file");

    let config: Config = toml::from_str(&config_content).expect("Failed to parse config file");

    log::info!("DBCLIENT version is {DBCLIENT_VERSION}. Application started with UI mode");

    let connections_client: Arc<dyn core::proto::connections::ConnectionsService + Send + Sync> =
        Arc::new(core::server::ConnectionsServer::new(
            config.connections.clone(),
        ));

    let db_objects_client: Arc<dyn core::proto::objects::ObjectsService + Send + Sync> =
        Arc::new(core::server::ObjectsServer::new(&config.connections));

    let queries_client: Arc<dyn core::proto::queries::QueriesService + Send + Sync> =
        Arc::new(core::server::QueriesServer::new(&config.connections));
    let app_state = Arc::new(Mutex::new(AppState {
        selected_connection: None,
        selected_object: None,
        selected_page: Page::ConnectionsList,
        focus_widget: ui4::app_state::Widget::default(),
    }));

    let (query_result_cmd_sender, query_result_cmd_reader) =
        tokio::sync::mpsc::channel::<QueryResultCmd>(10);

    element!(
        ui4::app_state::AppContainer(
            connections_client: Some(connections_client),
            objects_client: Some(db_objects_client),
            queries_client: Some(queries_client),
            state: app_state,
            query_result_cmd_pipe: Some(query_result_cmd_sender),
            query_result_cmd_receiver: Some(query_result_cmd_reader),
            notification_receiver: Some(notification_receiver),
        )
    )
    .render_loop()
    .await
    .context("app container execution error")
}

fn init_logger(
    ui_mode: bool,
    notification_sender: Option<tokio::sync::mpsc::Sender<Notification>>,
) -> Result<()> {
    let Some(mut logs_dir_path) = dirs::state_dir().or_else(|| dirs::data_local_dir()) else {
        return Err(anyhow::anyhow!("Can't determine state or data local dir"));
    };
    logs_dir_path.push("dbclient");
    logs_dir_path.push("logs");

    std::fs::create_dir_all(&logs_dir_path)?;

    let mut logger = Logger::try_with_env_or_str("info")?.write_mode(WriteMode::BufferAndFlush);

    let file_spec = FileSpec::default()
        .directory(logs_dir_path)
        .basename("dbclient")
        .suffix("log")
        .use_timestamp(false);

    if ui_mode {
        logger = logger.log_to_file_and_writer(
            file_spec,
            Box::new(VirtualLogger::new(notification_sender.unwrap())),
        );
    } else {
        logger = logger.log_to_file(file_spec);
    }

    logger.start()?;

    Ok(())
}
