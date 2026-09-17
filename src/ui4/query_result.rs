use std::{sync::Arc, time::Duration};

use iocraft::{
    AlignContent, AnyElement, Color, Edges, FlexDirection, Hooks, KeyCode, KeyEvent, KeyEventKind, Position, Props, component, components::{BorderStyle, Text, View}, element, hooks::{UseFuture, UseState, UseTerminalEvents},
};
use tokio::sync::Mutex;

use crate::{core::proto::{self, common::DbRecord}, ui4::{app_state::{AppState, QueryResultCmd}, forms::add_record_to_object::AddRecordToObjectForm}};

#[derive(Default, Props)]
pub struct QueryResultProps {
    pub queries_client: Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
}

#[component]
pub fn QueryResult(props: &QueryResultProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut to_visualize = hooks.use_state(|| DbRecord::default());
    let queries_client = props.queries_client.clone();
    let db_object_client = props.objects_client.clone();
    let mut selected_object = hooks.use_state(|| None);


    let mut render_add_record_to_object_widget = hooks.use_state(|| false);
    let mut add_record_to_object_widget_data_loaded = hooks.use_state(|| false);

    let state = props.state.clone();
    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await; // TODO: move to channels instead
                                                                 // of shared state
            let mut state = state.lock().await;
            match &state.query_result_cmd {
                Some(QueryResultCmd::ExecuteRawQuery) => {
                    todo!()
                },
                Some(QueryResultCmd::ListAllItemsFromObject) => {
                    match (&queries_client, &state.selected_connection, &state.selected_object) {
                        (Some(client), Some(selected_connection), Some(selected_object)) => {
                            let resp = client.list_all_items_from_object(proto::queries::ListAllItemsFromObjectRequest{
                                connection: Some(selected_connection.clone()),
                                object: Some(selected_object.clone())
                            }).await;
                            if let Ok(resp) = resp {
                                if resp.record.is_some() {
                                    to_visualize.set(resp.record.unwrap());
                                }
                            }
                        },
                        _ => {/* cannot perform operation */}
                    }
                },
                Some(QueryResultCmd::AddRecordToObject(record)) => {
                    match (&queries_client, &state.selected_connection, &state.selected_object) {
                        (Some(client), Some(selected_connection), Some(selected_object)) => {
                            let r = client.add_record_to_object(proto::queries::AddRecordToObjectRequest {
                                connection: Some(selected_connection.clone()),
                                object: Some(selected_object.clone()),
                                record: Some(record.clone())
                            }).await;
                            // TODO: log if error + restore records from object after operation done
                            // dbg!(r);
                        },
                        _ => {}
                    }
                    state.query_result_cmd = Some(QueryResultCmd::ClosePopup);
                    continue;
                    // render_add_record_to_object_widget.set(false);
                }
                Some(QueryResultCmd::ClosePopup) => {
                    selected_object.set(None);
                    render_add_record_to_object_widget.set(false);
                    state.focus_widget = crate::ui4::app_state::Widget::QueryResult;
                }
                _ => {}
            };
            state.query_result_cmd = None;
        }
    });

    let state_for_add_add_record_to_object_widget = props.state.clone();
    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;
            if render_add_record_to_object_widget.get() && !add_record_to_object_widget_data_loaded.get() {
                let state = state_for_add_add_record_to_object_widget.lock().await;
                match &db_object_client {
                    Some(cli) => {
                        let obj = cli.get_object_of_decriptor(proto::objects::GetObjectOfDecriptorRequest {
                            connection: state.selected_connection.clone(),
                            descriptor: state.selected_object.clone()
                        }).await;
                        if let Ok(r) = obj {
                            selected_object.set(r.object);
                        } else 
                        {
                            panic!("Error: {:?}", obj) // TODO: add log here if error
                        }
                    },
                    None => {},
                };
                add_record_to_object_widget_data_loaded.set(true);
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
            KeyCode::Char('a') => {
                render_add_record_to_object_widget.set(true);
                add_record_to_object_widget_data_loaded.set(false);
                state.focus_widget = crate::ui4::app_state::Widget::AnyPopup;
            },
            KeyCode::Char('h') => {
                state.focus_widget = crate::ui4::app_state::Widget::DbObjects;
            }
            _ => {}
        }
    });

    let form_state = props.state.clone();
    element! {
        View(
            width: 100pct,
            height: 100pct,
            justify_content: Some(AlignContent::Center),
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            #(
                match (render_add_record_to_object_widget.get(), selected_object.read().as_ref()) {
                    (true, Some(obj)) => {
                        element! {
                            View(
                                position: Position::Absolute,
                                width: 70pct,
                                height: 50pct,
                                top: 8,
                                left: 4,
                            ) {
                                AddRecordToObjectForm(
                                    state: form_state,
                                    object: obj.clone(),
                                )
                            }
                        }
                    },
                    _ => {
                        element! {
                            View {}
                        }
                    }
                }
            )
            #(
                match to_visualize.read().specification.as_ref() {
                    Some(proto::common::db_record::Specification::Postgres(postgres_record)) => {
                        element! {
                            View (width: 100pct, height: 100pct, flex_direction: FlexDirection::Column) {
                                #(
                                    match postgres_record.record.as_ref() {
                                        Some(proto::common::postgres_record::Record::Table(table)) => table
                                            .rows
                                            .iter()
                                            .enumerate()
                                            .map(|(i, row)| {
                                                element! {
                                                    View (flex_direction: FlexDirection::Column) {
                                                        #(if i == 0 {
                                                            element! {
                                                                View{
                                                                    #(row.columns.iter().map(|column| element! {
                                                                        View(width: 100pct, border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                                                                            Text(content: column.name.clone())
                                                                        }
                                                                    }))
                                                                }
                                                            }
                                                        } else {
                                                            element! {
                                                                View {}
                                                            }
                                                        })
                                                        View(background_color: if i % 2 == 0 { Some(Color::DarkGrey) } else { None }) {
                                                            #(row.values.iter().map(|value| element! {
                                                                View (width: 100pct) {
                                                                    Text(content: match &value.field {
                                                                        Some(proto::common::db_field::Field::Boolean(b)) => b.boolean.map_or(String::default(), |v| v.to_string()),
                                                                        Some(proto::common::db_field::Field::Str(str)) => str.str.as_ref().map_or(String::default(), |v| v.clone()),
                                                                        Some(proto::common::db_field::Field::I8(v)) => v.i8.map_or(String::default(), |v| v.to_string()),
                                                                        Some(proto::common::db_field::Field::I16(v)) => v.i16.map_or(String::default(), |v| v.to_string()),
                                                                        Some(proto::common::db_field::Field::I32(v)) => v.i32.map_or(String::default(), |v| v.to_string()),
                                                                        Some(proto::common::db_field::Field::I64(v)) => v.i64.map_or(String::default(), |v| v.to_string()),
                                                                        Some(proto::common::db_field::Field::Datetime(v)) => v.datetime.map_or(String::default(), |v| v.to_string()),
                                                                        _ => "<no value>".to_string()
                                                                    })
                                                                }
                                                            }))
                                                        }
                                                    }
                                                }
                                            })
                                            .collect(),
                                        _ => Vec::new(),
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
        }
    }
}
