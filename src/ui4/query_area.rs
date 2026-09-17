use std::sync::Arc;

use iocraft::{component, components::View, element, AnyElement, Props};
use tokio::sync::Mutex;

use crate::{
    core::proto,
    ui4::{app_state::AppState, db_objects::DbObjects, query_result::QueryResult},
};

#[derive(Default, Props)]
pub struct QueryAreaProps {
    pub objects_client: Option<Arc<dyn proto::objects::ObjectsService + Send + Sync>>,
    pub queries_client: Option<Arc<dyn proto::queries::QueriesService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
}

#[component]
pub fn QueryArea(props: &QueryAreaProps) -> impl Into<AnyElement<'static>> {
    element! {
        View (
            width: 100pct,
            height: 100pct
        ) {
            View (width: 30pct) {
                DbObjects(
                    objects_client: props.objects_client.clone(),
                    state: props.state.clone()
                )
            }

            View (width: 70pct) {
                QueryResult(
                    queries_client: props.queries_client.clone(),
                    objects_client: props.objects_client.clone(),
                    state: props.state.clone()
                )
            }
        }
    }
}
