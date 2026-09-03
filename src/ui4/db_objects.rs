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

fn collection_node(name: &str, children: Vec<FileTreeNode>) -> Option<FileTreeNode> {
    (!children.is_empty()).then(|| FileTreeNode::new(format!("▸ {name}"), children))
}

fn specification_node(specification: Specification) -> FileTreeNode {
    match specification {
        Specification::Redis(redis) => FileTreeNode::new(format!("Redis: {}", redis.name), vec![]),
        Specification::Postgres(postgres) => match postgres.object {
            Some(Object::Database(database)) => FileTreeNode::new(
                format!("⛁ {}", name(database.descriptor.as_ref(), |d| &d.name)),
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
                let tables = schema
                    .tables
                    .into_iter()
                    .map(|table| {
                        specification_node(Specification::Postgres(proto::common::PostgresObject {
                            object: Some(Object::Table(table)),
                        }))
                    })
                    .collect();
                let views = schema
                    .views
                    .into_iter()
                    .map(|view| FileTreeNode::new(format!("◉ {}", view.name), vec![]))
                    .collect();
                let materialized_views = schema
                    .materialized_views
                    .into_iter()
                    .map(|view| FileTreeNode::new(format!("◉ {}", view.name), vec![]))
                    .collect();
                let functions = schema
                    .functions
                    .into_iter()
                    .map(|function| FileTreeNode::new(format!("ƒ {}", function.name), vec![]))
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

                FileTreeNode::new(
                    format!(" {}", name(schema.descriptor.as_ref(), |d| &d.name)),
                    children,
                )
            }
            Some(Object::Table(table)) => {
                let columns = table
                    .columns
                    .into_iter()
                    .map(|column| FileTreeNode::new(format!("│ {}", column.name), vec![]))
                    .collect();
                let constraints = table
                    .constrains
                    .into_iter()
                    .map(|constraint| FileTreeNode::new(format!("⚿ {}", constraint.name), vec![]))
                    .collect();
                let indexes = table
                    .indexes
                    .into_iter()
                    .map(|index| FileTreeNode::new(format!("⌕ {}", index.name), vec![]))
                    .collect();

                let children = [
                    collection_node("Columns", columns),
                    collection_node("Constraints", constraints),
                    collection_node("Indexes", indexes),
                ]
                .into_iter()
                .flatten()
                .collect();

                FileTreeNode::new(
                    format!("▤ {}", name(table.descriptor.as_ref(), |d| &d.name)),
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
