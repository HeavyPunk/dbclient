use std::{sync::Arc, time::Duration};

use iocraft::{
    component,
    components::{BorderStyle, View},
    element,
    hooks::{UseFuture, UseState, UseTerminalEvents},
    AnyElement, Color, Hooks, KeyCode, KeyEvent, KeyEventKind, Props, TerminalEvent,
};
use tokio::sync::Mutex;

use crate::{
    core::proto::{
        self,
        common::{self, db_object::Specification, postgres_object::Object},
    },
    ui4::{
        app_state::{AppState, Page, QueryResultCmd, Widget},
        file_tree::{FileTree, FileTreeNode},
    },
};

type DbFileTreeNode = FileTreeNode<common::PostgresObjectDescriptor>;

#[derive(Default, Props)]
pub struct DbObjectsProps {
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
    pub state: Arc<Mutex<AppState>>,
}

fn name<T>(descriptor: Option<&T>, get_name: impl FnOnce(&T) -> &str) -> String {
    descriptor.map(get_name).unwrap_or("<unnamed>").to_string()
}

fn collection_node(name: &str, children: Vec<DbFileTreeNode>) -> Option<DbFileTreeNode> {
    (!children.is_empty()).then(|| DbFileTreeNode::new(format!("▸ {name}"), children))
}

fn specification_node(
    specification: Specification,
    state: Arc<Mutex<AppState>>,
    cmd_pipe: tokio::sync::mpsc::Sender<QueryResultCmd>,
) -> DbFileTreeNode {
    match specification {
        Specification::Redis(redis) => {
            DbFileTreeNode::new(format!("Redis: {}", redis.name), vec![])
        }
        Specification::Postgres(postgres) => match postgres.object {
            Some(Object::Database(database)) => {
                let mut node = DbFileTreeNode::new(
                    format!("⛁ {}", name(database.descriptor.as_ref(), |d| &d.name)),
                    database
                        .schemas
                        .into_iter()
                        .map(|schema| {
                            specification_node(
                                Specification::Postgres(proto::common::PostgresObject {
                                    object: Some(Object::Schema(schema)),
                                }),
                                state.clone(),
                                cmd_pipe.clone(),
                            )
                        })
                        .collect(),
                );
                if let Some(descriptor) = database.descriptor {
                    node = node.with_context(common::PostgresObjectDescriptor {
                        descriptor: Some(common::postgres_object_descriptor::Descriptor::Database(
                            descriptor,
                        )),
                    });
                }
                node
            }
            Some(Object::Schema(schema)) => {
                let schema_descriptor = schema.descriptor.clone();
                let tables = schema
                    .tables
                    .into_iter()
                    .map(|table| {
                        specification_node(
                            Specification::Postgres(proto::common::PostgresObject {
                                object: Some(Object::Table(table)),
                            }),
                            state.clone(),
                            cmd_pipe.clone(),
                        )
                    })
                    .collect();
                let views = schema
                    .views
                    .into_iter()
                    .map(|view| DbFileTreeNode::new(format!("◉ {}", view.name), vec![]))
                    .collect();
                let materialized_views = schema
                    .materialized_views
                    .into_iter()
                    .map(|view| DbFileTreeNode::new(format!("◉ {}", view.name), vec![]))
                    .collect();
                let functions = schema
                    .functions
                    .into_iter()
                    .map(|function| DbFileTreeNode::new(format!("ƒ {}", function.name), vec![]))
                    .collect();

                let children = [
                    collection_node("Tables", tables),
                    collection_node("Views", views),
                    collection_node("Materialized views", materialized_views),
                    collection_node("Functions", functions),
                ]
                .into_iter()
                .flatten()
                .collect();

                let mut node = DbFileTreeNode::new(
                    format!(" {}", name(schema.descriptor.as_ref(), |d| &d.name)),
                    children,
                );
                if let Some(descriptor) = schema_descriptor {
                    node = node.with_context(common::PostgresObjectDescriptor {
                        descriptor: Some(common::postgres_object_descriptor::Descriptor::Schema(
                            descriptor,
                        )),
                    });
                }
                node
            }
            Some(Object::Table(table)) => {
                let table_descriptor = table.descriptor.clone();
                let columns = table
                    .columns
                    .into_iter()
                    .map(|column| DbFileTreeNode::new(format!("│ {}", column.name), vec![]))
                    .collect();
                let constraints = table
                    .constrains
                    .into_iter()
                    .map(|constraint| DbFileTreeNode::new(format!("⚿ {}", constraint.name), vec![]))
                    .collect();
                let indexes = table
                    .indexes
                    .into_iter()
                    .map(|index| DbFileTreeNode::new(format!("⌕ {}", index.name), vec![]))
                    .collect();

                let children = [
                    collection_node("Columns", columns),
                    collection_node("Constraints", constraints),
                    collection_node("Indexes", indexes),
                ]
                .into_iter()
                .flatten()
                .collect();

                let mut node = DbFileTreeNode::new(
                    format!("▤ {}", name(table.descriptor.as_ref(), |d| &d.name)),
                    children,
                );
                if let Some(descriptor) = table_descriptor {
                    node = node
                        .with_context(common::PostgresObjectDescriptor {
                            descriptor: Some(
                                common::postgres_object_descriptor::Descriptor::Table(descriptor),
                            ),
                        })
                        .on_enter(move |ctx| {
                            {
                                let desc = ctx.descriptor;
                                let mut state =
                                    tokio::task::block_in_place(|| state.blocking_lock());
                                state.selected_object = Some(common::DbObjectDescriptor {
                                    descriptor: Some(
                                        common::db_object_descriptor::Descriptor::Postgres(desc),
                                    ),
                                });
                            }
                            // TODO: log if error
                            let _ = tokio::task::block_in_place(|| {
                                cmd_pipe.blocking_send(
                                    super::app_state::QueryResultCmd::ListAllItemsFromObject,
                                )
                            });
                        });
                }
                node
            }
            None => DbFileTreeNode::new("PostgreSQL: <unspecified>", vec![]),
        },
    }
}

fn object_node(
    object: common::DbObject,
    state: Arc<Mutex<AppState>>,
    cmd_pipe: tokio::sync::mpsc::Sender<QueryResultCmd>,
) -> Option<DbFileTreeNode> {
    object
        .specification
        .map(|spec| specification_node(spec, state, cmd_pipe))
}

#[component]
pub fn DbObjects(props: &DbObjectsProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let objects = hooks.use_state(Vec::<common::DbObject>::new);
    let selected_path = hooks.use_state(|| Arc::new(Mutex::new(None::<Vec<String>>)));
    let state = props.state.clone();
    let has_focus = hooks.use_state(|| false);

    let focus_state = props.state.clone();
    let mut focus = has_focus;
    hooks.use_future(async move {
        loop {
            let is_focused = focus_state.lock().await.focus_widget == Widget::DbObjects;
            if focus.get() != is_focused {
                focus.set(is_focused);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    if let Some(client) = props.objects_client.clone() {
        let mut objects = objects.clone();
        hooks.use_future(async move {
            let Some(connection) = state.lock().await.selected_connection.clone() else {
                return;
            };
            // TODO: log if error
            if let Ok(response) = client
                .get_objects_of_connection(proto::objects::GetObjectsOfConnectionRequest {
                    connection: Some(connection),
                })
                .await
            {
                objects.set(response.objects);
            }
        });
    }

    let Some(cmd_pipe) = props.cmd_pipe.as_ref() else {
        return element! {
            View
        };
    };

    let nodes: Vec<DbFileTreeNode> = objects
        .read()
        .iter()
        .cloned()
        .filter_map(|obj| object_node(obj, props.state.clone(), cmd_pipe.clone()))
        .collect();

    let state = props.state.clone();
    hooks.use_local_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        let mut state = tokio::task::block_in_place(|| state.blocking_lock());
        if state.focus_widget != Widget::DbObjects {
            return;
        }

        match code {
            KeyCode::Char('l') => {
                state.focus_widget = Widget::QueryResult;
            }
            KeyCode::Esc => {
                state.selected_page = Page::ConnectionsList;
            }
            _ => {}
        };
    });

    element! {
        View(width: 100pct, height: 100pct, border_style: BorderStyle::Round, border_color: if focus.get() { Color::Yellow } else { Color::Cyan }) {
            FileTree::<common::PostgresObjectDescriptor>(
                nodes: nodes,
                selected_path: selected_path.read().clone(),
                has_focus: has_focus.get(),
            )
        }
    }
}
