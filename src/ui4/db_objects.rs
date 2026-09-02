use std::sync::Arc;

use iocraft::{
    component,
    components::{BorderStyle, View},
    element,
    hooks::{UseFuture, UseState},
    AnyElement, Color, Hooks, Props,
};
use tokio::sync::Mutex;

use crate::{
    core::proto::{
        self,
        common::{self, db_object::Specification, postgres_object::Object},
    },
    ui4::{
        app_state::AppState,
        file_tree::{FileTree, FileTreeNode},
    },
};

#[derive(Default, Props)]
pub struct DbObjectsProps {
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
}

fn name<T>(descriptor: Option<&T>, get_name: impl FnOnce(&T) -> &str) -> String {
    descriptor.map(get_name).unwrap_or("<unnamed>").to_string()
}

fn specification_node(specification: Specification) -> FileTreeNode {
    match specification {
        Specification::Redis(redis) => FileTreeNode::new(format!("Redis: {}", redis.name), vec![]),
        Specification::Postgres(postgres) => match postgres.object {
            Some(Object::Database(database)) => FileTreeNode::new(
                format!(
                    "Database: {}",
                    name(database.descriptor.as_ref(), |d| &d.name)
                ),
                database
                    .schemas
                    .into_iter()
                    .map(|schema| {
                        specification_node(Specification::Postgres(proto::common::PostgresObject {
                            object: Some(Object::Schema(schema)),
                        }))
                    })
                    .collect(),
            ),
            Some(Object::Schema(schema)) => {
                let mut children = schema
                    .tables
                    .into_iter()
                    .map(|table| {
                        specification_node(Specification::Postgres(proto::common::PostgresObject {
                            object: Some(Object::Table(table)),
                        }))
                    })
                    .collect::<Vec<_>>();
                children.extend(
                    schema
                        .views
                        .into_iter()
                        .map(|view| FileTreeNode::new(format!("View: {}", view.name), vec![])),
                );
                children.extend(schema.materialized_views.into_iter().map(|view| {
                    FileTreeNode::new(format!("Materialized view: {}", view.name), vec![])
                }));
                children.extend(schema.functions.into_iter().map(|function| {
                    FileTreeNode::new(format!("Function: {}", function.name), vec![])
                }));
                FileTreeNode::new(
                    format!("Schema: {}", name(schema.descriptor.as_ref(), |d| &d.name)),
                    children,
                )
            }
            Some(Object::Table(table)) => {
                let mut children = table
                    .columns
                    .into_iter()
                    .map(|column| FileTreeNode::new(format!("Column: {}", column.name), vec![]))
                    .collect::<Vec<_>>();
                children.extend(table.constrains.into_iter().map(|constraint| {
                    FileTreeNode::new(format!("Constraint: {}", constraint.name), vec![])
                }));
                children.extend(
                    table
                        .indexes
                        .into_iter()
                        .map(|index| FileTreeNode::new(format!("Index: {}", index.name), vec![])),
                );
                FileTreeNode::new(
                    format!("Table: {}", name(table.descriptor.as_ref(), |d| &d.name)),
                    children,
                )
            }
            None => FileTreeNode::new("PostgreSQL: <unspecified>", vec![]),
        },
    }
}

fn object_node(object: common::DbObject) -> Option<FileTreeNode> {
    object.specification.map(specification_node)
}

#[component]
pub fn DbObjects(props: &DbObjectsProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let objects = hooks.use_state(Vec::<common::DbObject>::new);
    let selected_path = hooks.use_state(|| Arc::new(Mutex::new(None::<Vec<String>>)));
    let state = props.state.clone();

    if let Some(client) = props.objects_client.clone() {
        let mut objects = objects.clone();
        hooks.use_future(async move {
            let Some(connection) = state.lock().await.selected_connection.clone() else {
                return;
            };
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

    let nodes: Vec<FileTreeNode> = objects
        .read()
        .iter()
        .cloned()
        .filter_map(object_node)
        .collect();
    element! {
        View(width: 100pct, height: 100pct, border_style: BorderStyle::Round, border_color: Color::Cyan) {
            FileTree(nodes: nodes, selected_path: selected_path.read().clone())
        }
    }
}
