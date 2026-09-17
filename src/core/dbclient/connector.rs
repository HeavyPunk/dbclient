use crate::core::proto;

#[derive(Debug)]
pub enum ConnectorError {
    InvalidRequest(&'static str),
    PostgresError(tokio_postgres::Error),
}

pub type GetObjectsResult = proto::common::DbObject;
pub type GetObjectRequest = proto::common::DbObjectDescriptor;
pub type GetObjectResult = proto::common::DbObject;
pub type ListAllItemsFromObjectRequest = proto::common::DbObjectDescriptor;
pub type ListAllItemsFromObjectResult = proto::common::DbRecord;

pub struct AddRecordToObjectRequest {
    pub descriptor: proto::common::DbObjectDescriptor,
    pub record: proto::common::DbRecord,
}

#[async_trait::async_trait]
pub trait Connector: Send {
    async fn get_objects(&mut self) -> Result<GetObjectsResult, ConnectorError>;
    async fn get_object(&mut self, req: GetObjectRequest) -> Result<GetObjectResult, ConnectorError>;
    async fn list_all_items_from_object(&mut self, req: ListAllItemsFromObjectRequest) -> Result<ListAllItemsFromObjectResult, ConnectorError>;

    async fn add_record_to_object(&mut self, req: AddRecordToObjectRequest) -> Result<(), ConnectorError>;
}
