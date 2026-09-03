use std::{collections::HashMap, fmt};

use crate::config::Connection;
use crate::core::dbclient::connector::{Connector, ConnectorError};
use crate::core::dbclient::postgresql::connector_impl::PostgresConnector;
use crate::core::{
    dbclient::{
        dummy::DummyFetcher,
        fetcher::{FetchRequest, FetchResult, Fetcher, FetcherError},
        postgresql::{PostgresConfig, PostgresFetcher},
        query_builder::QueryElement,
        redis::{RedisConfig, RedisFetcher},
    },
    proto::{
        self,
        common::DbObject,
        connections::{GetAvailableConnectionsRequest, GetAvailableConnectionsResponse},
        objects::{
            AddObjectToConnectionRequest, AddObjectToConnectionResponse,
            GetObjectsOfConnectionRequest, GetObjectsOfConnectionResponse,
        },
    },
};

pub struct ConnectionsServer {
    conns: Vec<Connection>,
}

impl ConnectionsServer {
    pub fn new(connections: Vec<Connection>) -> Self {
        Self { conns: connections }
    }
}

#[async_trait::async_trait]
impl super::proto::connections::ConnectionsService for ConnectionsServer {
    async fn get_available_list(
        &self,
        _: GetAvailableConnectionsRequest,
    ) -> ::anyhow::Result<GetAvailableConnectionsResponse> {
        Ok(GetAvailableConnectionsResponse {
            connections: self
                .conns
                .iter()
                .map(|c| proto::connections::Connection { id: c.name.clone() })
                .collect(),
        })
    }
}

pub struct ObjectsServer {
    conns: HashMap<String, Connection>,
}

#[derive(Debug)]
enum ObjectsServerErrors {
    ValidationError(&'static str),
    ConnectionNotFound,
    FetcherError(FetcherError),
    ConnectorError(ConnectorError),
}

impl fmt::Display for ObjectsServerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectsServerErrors::ValidationError(e) => write!(f, "validation error: {}", e),
            ObjectsServerErrors::ConnectionNotFound => write!(f, "connection not found"),
            ObjectsServerErrors::FetcherError(_) => write!(f, "fetcher error"),
            ObjectsServerErrors::ConnectorError(_) => write!(f, "connector error"),
        }
    }
}

impl From<FetcherError> for ObjectsServerErrors {
    fn from(value: FetcherError) -> Self {
        return Self::FetcherError(value);
    }
}

impl std::error::Error for ObjectsServerErrors {}

impl ObjectsServer {
    pub fn new(connections: Vec<Connection>) -> Self {
        let mut conns = HashMap::new();
        for conn in connections {
            conns.insert(conn.name.clone(), conn);
        }
        Self { conns: conns }
    }

    fn resolve_connector(conn: &Connection) -> Box<dyn Connector> {
        let conn: Box<_> = match conn.connection_type {
            crate::config::ConnectionType::Redis => todo!(),
            crate::config::ConnectionType::Postgres => Box::new(PostgresConnector {
                config: super::dbclient::postgresql::connector_impl::PostgresConfig {
                    uri: conn.connection_string.clone(),
                },
            }),
            crate::config::ConnectionType::MySql => todo!(),
        };

        return conn;
    }

    fn resolve_fetcher(conn: &Connection) -> Box<dyn Fetcher> {
        let fetcher: Box<dyn Fetcher> = match conn.connection_type {
            crate::config::ConnectionType::Redis => Box::new(RedisFetcher {
                config: RedisConfig {
                    uri: conn.connection_string.clone(),
                },
            }),
            crate::config::ConnectionType::Postgres => Box::new(PostgresFetcher {
                config: PostgresConfig {
                    uri: conn.connection_string.clone(),
                },
            }),
            crate::config::ConnectionType::MySql => Box::new(DummyFetcher::new()),
        };
        return fetcher;
    }

    async fn list_objects(conn: &Connection) -> Result<Vec<DbObject>, ConnectorError> {
        let mut conn = ObjectsServer::resolve_connector(conn);
        let obj = conn.get_objects().await?;
        Ok(vec![obj])
    }

    // fn add_object(conn: &Connection, obj: DbObject) -> Result<(), FetcherError> {
    //     let mut fetcher = ObjectsServer::resolve_fetcher(conn);
    //     let path = {
    //         let mut p = vec![];
    //         p.push(obj.id.clone());
    //         let mut current = obj.prev.as_deref();
    //         while let Some(parent) = current {
    //             p.push(parent.id.clone());
    //             current = parent.prev.as_deref();
    //         }
    //         p.reverse();
    //         p.join("/")
    //     };
    //
    //     let query = FetchRequest {
    //         query: vec![QueryElement::AddDatabaseObject(path, obj.r#type, obj.name)],
    //         limit: usize::MAX,
    //     };
    //     fetcher.fetch(&query)?;
    //     Ok(())
    // }
}

#[async_trait::async_trait]
impl super::proto::objects::ObjectsService for ObjectsServer {
    async fn get_objects_of_connection(
        &self,
        request: GetObjectsOfConnectionRequest,
    ) -> ::anyhow::Result<GetObjectsOfConnectionResponse> {
        if let Some(connection) = request.connection {
            let conn = self.conns.get(&connection.id);
            if let Some(conn) = conn {
                let objs = match Self::list_objects(conn).await {
                    Ok(v) => v,
                    Err(e) => return Err(ObjectsServerErrors::ConnectorError(e))?,
                };
                Ok(GetObjectsOfConnectionResponse { objects: objs })
            } else {
                Err(ObjectsServerErrors::ConnectionNotFound)?
            }
        } else {
            Err(ObjectsServerErrors::ValidationError(
                "connection value is required",
            ))?
        }
    }

    async fn add_object_to_connection(
        &self,
        request: AddObjectToConnectionRequest,
    ) -> ::anyhow::Result<AddObjectToConnectionResponse> {
        // match request {
        //     AddObjectToConnectionRequest {
        //         connection: Some(conn),
        //         object: Some(obj),
        //     } => {
        //         let conn = self.conns.get(&conn.id);
        //         if conn.is_none() {
        //             return Err(ObjectsServerErrors::ConnectionNotFound)?;
        //         }
        //         if let Err(e) = Self::add_object(&conn.unwrap(), obj) {
        //             return Err(ObjectsServerErrors::FetcherError(e))?;
        //         }
        //         return Ok(AddObjectToConnectionResponse {});
        //     }
        //     _ => Err(ObjectsServerErrors::ValidationError(
        //         "connection and object are required",
        //     ))?,
        // }
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
