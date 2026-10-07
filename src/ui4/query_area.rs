use std::sync::Arc;

use iocraft::{component, components::View, element, hooks::UseState, AnyElement, Hooks, Props};
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
}

#[component]
pub async fn QueryArea(
    props: &mut QueryAreaProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let chan = hooks.use_state(|| {
        let (query_result_cmd_sender, query_result_cmd_reader) =
            tokio::sync::mpsc::channel::<QueryResultCmd>(10);
        return (
            query_result_cmd_sender,
            Arc::new(Mutex::new(query_result_cmd_reader)),
        );
    });

    element! {
        View (
            width: 100pct,
            height: 100pct
        ) {
            View (width: 30pct) {
                DbObjects(
                    objects_client: props.objects_client.clone(),
                    cmd_pipe: Some(chan.read().0.clone()),
                    state: props.state.clone()
                )
            }

            View (width: 70pct) {
                QueryResult(
                    queries_client: props.queries_client.clone(),
                    objects_client: props.objects_client.clone(),
                    state: props.state.clone(),
                    cmd_pipe: Some(chan.read().0.clone()),
                    cmd_receiver: Some(chan.read().1.clone()),
                )
            }
        }
    }
}
