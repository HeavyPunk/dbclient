use std::{sync::Arc, time::Duration};

use iocraft::{
    AlignContent, AnyElement, Color, Edges, FlexDirection, Hooks, Props, component, components::{BorderStyle, Text, View}, element, hooks::{UseFuture, UseState},
};
use tokio::sync::Mutex;

use crate::{core::proto::{self, common::DbRecord}, ui4::app_state::{AppState, QueryResultCmd}};

#[derive(Default, Props)]
pub struct QueryResultProps {
    pub queries_client: Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
}

#[component]
pub fn QueryResult(props: &QueryResultProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut to_visualize = hooks.use_state(|| DbRecord::default());
    let state = props.state.clone();
    let client = props.queries_client.clone();
    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await; // TODO: move to channels instead
                                                                 // of shared state
            let mut state = state.lock().await;
            match state.query_result_cmd {
                Some(QueryResultCmd::ExecuteRawQuery) => {
                    todo!()
                },
                Some(QueryResultCmd::ListAllItemsFromObject) => {
                    match (&client, &state.selected_connection, &state.selected_object) {
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
                _ => {}
            };
            state.query_result_cmd = None;
        }
    });

    element! {
        View(
            width: 100pct,
            height: 100pct,
            justify_content: Some(AlignContent::Center),
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
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
                                                                        Some(proto::common::db_field::Field::Boolean(b)) => b.to_string(),
                                                                        Some(proto::common::db_field::Field::Str(str)) => str.clone(),
                                                                        Some(proto::common::db_field::Field::I8(v)) => v.to_string(),
                                                                        Some(proto::common::db_field::Field::I16(v)) => v.to_string(),
                                                                        Some(proto::common::db_field::Field::I32(v)) => v.to_string(),
                                                                        Some(proto::common::db_field::Field::I64(v)) => v.to_string(),
                                                                        Some(proto::common::db_field::Field::Datetime(v)) => v.to_string(),
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
