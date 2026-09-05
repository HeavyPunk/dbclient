use crate::core::proto;

#[derive(Debug)]
pub enum ConnectorError {
    PostgresError(tokio_postgres::Error),
}

pub type GetObjectsResult = proto::common::DbObject;
pub type ListAllItemsFromObjectResult = proto::common::DbRecord;

#[async_trait::async_trait]
pub trait Connector: Send {
    async fn get_objects(&mut self) -> Result<GetObjectsResult, ConnectorError>;
    async fn list_all_items_from_object(&mut self) -> Result<ListAllItemsFromObjectResult, ConnectorError>;
}
