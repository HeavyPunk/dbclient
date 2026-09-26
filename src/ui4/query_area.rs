use std::sync::Arc;

use iocraft::{component, components::View, element, AnyElement, Props};
use tokio::sync::Mutex;

use crate::{
    core::proto,
    ui4::{
        app_state::{AppState, QueryResultCmd},
        db_objects::DbObjects,
        query_result::QueryResult,
    },
};

#[derive(Default, Props)]
pub struct QueryAreaProps {
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub queries_client: Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
    pub query_result_cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
    pub query_result_cmd_receiver: Option<tokio::sync::mpsc::Receiver<QueryResultCmd>>,
}

#[component]
pub fn QueryArea(props: &mut QueryAreaProps) -> impl Into<AnyElement<'static>> {
    element! {
        View (
            width: 100pct,
            height: 100pct
        ) {
            View (width: 30pct) {
                DbObjects(
                    objects_client: props.objects_client.clone(),
                    cmd_pipe: props.query_result_cmd_pipe.clone(),
                    state: props.state.clone()
                )
            }

            View (width: 70pct) {
                QueryResult(
                    queries_client: props.queries_client.clone(),
                    objects_client: props.objects_client.clone(),
                    state: props.state.clone(),
                    cmd_pipe: props.query_result_cmd_pipe.clone(),
                    cmd_receiver: props.query_result_cmd_receiver.take(),
                )
            }
        }
    }
}
