use std::{sync::Arc, time::Duration};

use iocraft::{
    component,
    components::{Text, View},
    element,
    hooks::{UseFuture, UseState, UseTerminalEvents, UseTerminalSize},
    AnyElement, Hooks, Props,
};
use tokio::sync::Mutex;

use crate::{
    core::proto,
    ui4::{connections_list::ConnectionsList, query_area::QueryArea},
};

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    ConnectionsList,
    QueryArea,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Widget {
    DbObjects,
    QueryResult,
    AnyPopup,
}

impl Default for Widget {
    fn default() -> Self {
        Self::DbObjects
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::ConnectionsList
    }
}

#[derive(Clone, PartialEq)]
pub enum QueryResultCmd {
    ExecuteRawQuery,
    ListAllItemsFromObject,
    AddRecordToObject(proto::common::DbRecord),
    ClosePopup
}

#[derive(Default)]
pub struct AppState {
    pub selected_connection: Option<proto::connections::Connection>,
    pub selected_object: Option<proto::common::DbObjectDescriptor>,
    pub query_result_cmd: Option<QueryResultCmd>,
    pub selected_page: Page,
    pub focus_widget: Widget,
}

#[derive(Default, Props)]
pub struct AppContainerProps {
    pub connections_client: Option<Arc<dyn proto::connections::ConnectionsService + Send + Sync>>,
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub queries_client: Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
}

#[component]
pub fn AppContainer(props: &AppContainerProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, mut height) = hooks.use_terminal_size();
    height = height - 1;

    let state = props.state.clone();
    let mut page = hooks.use_state(|| Page::default());

    let (width, mut height) = hooks.use_terminal_size();
    height = height - 1;

    hooks.use_future(async move {
        loop {
            let selected_page = state.lock().await.selected_page;

            if page.get() != selected_page {
                page.set(selected_page);
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    element! {
        View
        {
            #(match page.get() {
                Page::ConnectionsList => element! {
                    View (
                        width,
                        height,
                    ){
                        ConnectionsList(
                            connections_client: props.connections_client.clone(),
                            state: props.state.clone()
                        )
                    }
                },
                Page::QueryArea => element! {
                    View (
                        width,
                        height,
                    ) {
                        QueryArea(
                            queries_client: props.queries_client.clone(),
                            objects_client: props.objects_client.clone(),
                            state: props.state.clone()
                        )
                    }
                },
            })
        }
    }
}
