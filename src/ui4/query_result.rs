use std::{sync::Arc, time::Duration};

use iocraft::{
    component,
    components::{BorderStyle, Text, View},
    element,
    hooks::{State, UseFuture, UseState, UseTerminalEvents},
    AlignContent, AnyElement, Color, Hooks, KeyCode, KeyEvent, KeyEventKind, Position, Props,
};
use tokio::sync::Mutex;

use crate::{
    core::proto::{
        self,
        common::{db_field::Field, DbRecord},
    },
    ui4::{
        app_state::{AppState, Page, QueryResultCmd},
        components::table_view::{table_event_handler, TableView},
        forms::{
            add_record_to_object::AddRecordToObjectForm,
            update_record_of_object::UpdateRecordOfObject,
        },
    },
};

fn postgres_field_from_common(field: &Field) -> proto::postgresql::DbField {
    match field {
        Field::Postgres(field) => field.clone(),
    }
}

#[derive(PartialEq, Clone)]
enum Popup {
    AddRecordToObject(bool),
    UpdateRecordOfObject(proto::common::DbRecord),
}

#[derive(Default, Props)]
pub struct QueryResultProps {
    pub queries_client: Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
    pub cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
    pub cmd_receiver: Option<Arc<Mutex<tokio::sync::mpsc::Receiver<QueryResultCmd>>>>,
}

#[component]
pub fn QueryResult(
    props: &mut QueryResultProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut to_visualize = hooks.use_state(|| DbRecord::default());
    let queries_client = props.queries_client.clone();
    let db_object_client = props.objects_client.clone();
    let mut selected_object = hooks.use_state(|| None);

    let mut render_popup: iocraft::prelude::State<Option<Popup>> = hooks.use_state(|| None);

    let state = props.state.clone();
    let Some(self_cmd_sender) = props.cmd_pipe.clone() else {
        log::warn!("cmd_pipe is undefined, sending commands to pipe is impossible");
        return element! {
            View
        };
    };
    let Some(cmd_receiver) = props.cmd_receiver.clone() else {
        log::warn!("cmd_receiver is undefined, getting commands from pipe is impossible");
        return element! {
            View
        };
    };
    hooks.use_future(async move {
        loop {
            let cmd = {
                let mut receiver = cmd_receiver.lock().await;
                receiver.recv().await
            };
            let res = run_cmd(
                &cmd,
                &state,
                &queries_client,
                &mut to_visualize,
                &mut render_popup,
                &mut selected_object,
                &self_cmd_sender,
            )
            .await;
            if let Err(e) = res {
                if let Some(cmd) = cmd {
                    log::warn!("Error on performing cmd {cmd}: {e}");
                } else {
                    log::warn!("Error on performing cmd: {e}")
                }
            }
        }
    });

    let state_for_add_add_record_to_object_widget = props.state.clone();
    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let popup = {
                let r = render_popup.read();
                r.clone()
            };
            match popup {
                Some(Popup::AddRecordToObject(is_data_loaded)) if !is_data_loaded => {
                    let state = state_for_add_add_record_to_object_widget.lock().await;
                    match &db_object_client {
                        Some(cli) => {
                            let obj = cli
                                .get_object_of_decriptor(
                                    proto::objects::GetObjectOfDecriptorRequest {
                                        connection: state.selected_connection.clone(),
                                        descriptor: state.selected_object.clone(),
                                    },
                                )
                                .await;
                            match obj {
                                Ok(r) => selected_object.set(r.object),
                                Err(e) => log::error!(target: "query_result", "Error on get object of descriptor: {e}")
                            };
                        }
                        None => {}
                    };
                    render_popup.set(Some(Popup::AddRecordToObject(true)));
                }
                _ => {}
            }
        }
    });

    let state = props.state.clone();
    hooks.use_local_terminal_events(move |event| {
        let iocraft::TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind != KeyEventKind::Press {
            return;
        }
        let mut state = tokio::task::block_in_place(|| state.blocking_lock());
        if state.focus_widget != crate::ui4::app_state::Widget::QueryResult {
            return;
        }

        match code {
            KeyCode::Char('h') => {
                state.focus_widget = crate::ui4::app_state::Widget::DbObjects;
            }
            KeyCode::Esc => {
                state.selected_page = Page::ConnectionsList;
            }
            _ => {}
        }
    });

    element! {
        View(
            width: 100pct,
            height: 100pct,
            position: Position::Relative,
            justify_content: Some(AlignContent::Center),
            border_style: BorderStyle::Round,
            border_color: if tokio::task::block_in_place(|| props.state.blocking_lock().focus_widget == crate::ui4::app_state::Widget::QueryResult) { Color::Yellow } else { Color::Cyan },
        ) {
            #(
                match to_visualize.read().specification.as_ref() {
                    Some(proto::common::db_record::Specification::Postgres(postgres_record)) => {
                        element! {
                            View (width: 100pct, height: 100pct) {
                                #(
                                    match postgres_record.record.as_ref() {
                                        Some(proto::postgresql::postgres_record::Record::Table(table)) => {
                                            let table_callback_state_on_add = props.state.clone();
                                            let table_callback_state_on_update = props.state.clone();
                                            let cmd_sender = props.cmd_pipe.clone();
                                            element! {
                                                TableView(
                                                    columns: table.columns.iter().map(|column| column.name.clone()).collect::<Vec<_>>(),
                                                    rows: table.rows.iter().map(|row| row.values.iter().filter_map(|value| value.field.clone().map(|field| Field::Postgres(proto::postgresql::DbField { field: Some(field) }))).collect::<Vec<Field>>()).collect::<Vec<Vec<Field>>>(),
                                                    has_focus: tokio::task::block_in_place(|| props.state.blocking_lock().focus_widget == crate::ui4::app_state::Widget::QueryResult),
                                                    on_add_row: Some(table_event_handler(move |_, _| {
                                                        let mut state_guard = tokio::task::block_in_place(|| table_callback_state_on_add.blocking_lock());
                                                        render_popup.set(Some(Popup::AddRecordToObject(false)));
                                                        state_guard.focus_widget = crate::ui4::app_state::Widget::AnyPopup;
                                                    })),
                                                    on_update_row: Some(table_event_handler(move |columns, row| {
                                                        let mut state_guard = tokio::task::block_in_place(|| table_callback_state_on_update.blocking_lock());
                                                        let initial_record = proto::common::DbRecord {
                                                            specification: Some(proto::common::db_record::Specification::Postgres(proto::postgresql::PostgresRecord {
                                                                record: Some(proto::postgresql::postgres_record::Record::Table(proto::postgresql::PostgresRecordTable {
                                                                    columns: columns.iter().map(|c| proto::postgresql::PostgresTableColumn { name: c.clone(), field: None }).collect(),
                                                                    rows: vec![proto::postgresql::PostgresRecordTableRow {
                                                                        values: row.iter().map(postgres_field_from_common).collect()
                                                                    }]
                                                                }))
                                                            }))
                                                        };
                                                        render_popup.set(Some(Popup::UpdateRecordOfObject(initial_record)));
                                                        state_guard.focus_widget = crate::ui4::app_state::Widget::AnyPopup;
                                                    })),
                                                    on_delete_row: Some(table_event_handler(move |columns, row| {
                                                        let cmd = QueryResultCmd::RemoveRecordFromObject(proto::common::DbRecord {
                                                            specification: Some(proto::common::db_record::Specification::Postgres(proto::postgresql::PostgresRecord {
                                                                record: Some(proto::postgresql::postgres_record::Record::Table(proto::postgresql::PostgresRecordTable {
                                                                    columns: columns.iter().map(|c| proto::postgresql::PostgresTableColumn { name: c.clone(), field: None }).collect(),
                                                                    rows: vec![proto::postgresql::PostgresRecordTableRow {
                                                                        values: row.iter().map(postgres_field_from_common).collect()
                                                                    }]
                                                                }))
                                                            }))
                                                        });
                                                        tokio::task::block_in_place(|| cmd_sender.as_ref().and_then(|ch| Some(ch.blocking_send(cmd))));
                                                    }))
                                                )
                                            }
                                        },
                                        _ => element! {
                                            TableView(
                                                columns: Vec::<String>::new(),
                                                rows: Vec::<Vec<Field>>::new(),
                                                has_focus: false,
                                            )
                                        },
                                    }
                                )
                            }
                        }
                    }
                    _ =>
                        element! {
                            View {
                                Text(content: "Nothing to display")
                            }
                        },
                }
            )
            #(
                match (render_popup.read().as_ref(), selected_object.read().as_ref()) {
                    (Some(Popup::AddRecordToObject(true)), Some(obj)) => {
                        element! {
                            View(
                                position: Position::Absolute,
                                width: 70pct,
                                height: 50pct,
                                top: 8,
                                left: 4,
                                background_color: Color::Black,
                            ) {
                                AddRecordToObjectForm(
                                    object: obj.clone(),
                                    cmd_pipe: props.cmd_pipe.clone(),
                                )
                            }
                        }
                    },
                    (Some(Popup::UpdateRecordOfObject(initial_record)), _) => {
                        element! {
                            View(
                                position: Position::Absolute,
                                width: 70pct,
                                height: 50pct,
                                top: 8,
                                left: 4,
                                background_color: Color::Black,
                            ) {
                                UpdateRecordOfObject(
                                    initial_record: initial_record.clone(),
                                    cmd_pipe: props.cmd_pipe.clone(),
                                )
                            }
                        }
                    }
                    _ => {
                        element! {
                            View {}
                        }
                    }
                }
            )
        }
    }
}

async fn run_cmd(
    cmd: &Option<QueryResultCmd>,
    state: &Mutex<AppState>,
    queries_client: &Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    to_visualize: &mut State<DbRecord>,
    render_popup: &mut State<Option<Popup>>,
    selected_object: &mut State<Option<proto::common::DbObject>>,
    self_cmd_sender: &tokio::sync::mpsc::Sender<QueryResultCmd>,
) -> anyhow::Result<()> {
    return match cmd {
        Some(QueryResultCmd::ExecuteRawQuery) => {
            Err(anyhow::anyhow!("operation not yet implemented"))
        }
        Some(QueryResultCmd::ListAllItemsFromObject) => {
            let (connection, object) = {
                let state = state.lock().await;
                (
                    state.selected_connection.clone(),
                    state.selected_object.clone(),
                )
            };
            match (&queries_client, &connection, &object) {
                (Some(client), Some(selected_connection), Some(selected_object)) => {
                    match client.list_all_items_from_object(
                        proto::queries::ListAllItemsFromObjectRequest {
                            connection: Some(selected_connection.clone()),
                            object: Some(selected_object.clone()),
                        }).await {
                        Ok(resp) => {
                            if resp.record.is_some() {
                                to_visualize.set(resp.record.unwrap());
                            }
                            Ok(())
                        },
                        Err(e) => Err(e),
                    }
                }
                _ => Err(anyhow::anyhow!("cannot performing operation, client, selected_connection or selected_object is undefined"))
            }.map_err(|e| anyhow::anyhow!("Error on list all items from object: {e}"))
        }
        Some(QueryResultCmd::AddRecordToObject(record)) => {
            let (connection, object) = {
                let state = state.lock().await;
                (
                    state.selected_connection.clone(),
                    state.selected_object.clone(),
                )
            };
            match (&queries_client, &connection, &object) {
                (Some(client), Some(selected_connection), Some(selected_object)) => {
                    client.add_record_to_object(proto::queries::AddRecordToObjectRequest {
                        connection: Some(selected_connection.clone()),
                        object: Some(selected_object.clone()),
                        record: Some(record.clone()),
                    }).await
                }
                _ => Err(anyhow::anyhow!("cannot performing operation, client, selected_connection or selected_object is undefined"))
            }.map_err(|e| anyhow::anyhow!("Error on add record to object: {e}"))?;

            self_cmd_sender.send(QueryResultCmd::ClosePopup).await?;
            self_cmd_sender
                .send(QueryResultCmd::ListAllItemsFromObject)
                .await?;
            Ok(())
        }
        Some(QueryResultCmd::RemoveRecordFromObject(record)) => {
            let (connection, object) = {
                let state = state.lock().await;
                (
                    state.selected_connection.clone(),
                    state.selected_object.clone(),
                )
            };
            match (&queries_client, &connection, &object) {
                (Some(client), Some(selected_connection), Some(selected_object)) => {
                    client.remove_record_from_object(
                        proto::queries::RemoveRecordFromObjectRequest {
                            connection: Some(selected_connection.clone()),
                            object: Some(selected_object.clone()),
                            record: Some(record.clone()),
                        }).await
                }
                _ => Err(anyhow::anyhow!("cannot performing operation, client, selected_connection or selected_object is undefined"))
            }.map_err(|e| anyhow::anyhow!("Error on remove record from object: {e}"))?;

            let _ = self_cmd_sender
                .send(QueryResultCmd::ListAllItemsFromObject)
                .await?;
            Ok(())
        }
        Some(QueryResultCmd::UpdateRecordOfObject(old_record, new_record)) => {
            let (connection, object) = {
                let state = state.lock().await;
                (
                    state.selected_connection.clone(),
                    state.selected_object.clone(),
                )
            };
            match (&queries_client, &connection, &object) {
                (Some(client), Some(selected_connection), Some(selected_object)) => {
                    client.update_record_of_object(
                        proto::queries::UpdateRecordOfObjectRequest {
                            connection: Some(selected_connection.clone()),
                            object: Some(selected_object.clone()),
                            old_record: Some(old_record.clone()),
                            new_record: Some(new_record.clone()),
                        }).await
                }
                _ => Err(anyhow::anyhow!("cannot performing operation, client, selected_connection or selected_object is undefined"))
            }.map_err(|e| anyhow::anyhow!("Error on update record of object: {e}"))?;
            self_cmd_sender.send(QueryResultCmd::ClosePopup).await?;
            self_cmd_sender
                .send(QueryResultCmd::ListAllItemsFromObject)
                .await?;
            Ok(())
        }
        Some(QueryResultCmd::OpenAddRecordPopup) => {
            render_popup.set(Some(Popup::AddRecordToObject(false)));
            Ok(())
        }
        Some(QueryResultCmd::ClosePopup) => {
            // NOTE: wait until other widgets hooks observe event as trash before enable
            // QueryResult widget as event consumer
            // see race condition problem here in hooks.use_local_terminal_events
            tokio::time::sleep(Duration::from_millis(100)).await;
            selected_object.set(None);
            render_popup.set(None);
            let mut state = state.lock().await;
            state.focus_widget = crate::ui4::app_state::Widget::QueryResult;
            Ok(())
        }
        _ => Ok(()),
    };
}
