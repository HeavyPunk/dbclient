use std::collections::HashMap;

use tokio_postgres::NoTls;

use crate::core::{
    dbclient::connector::{Connector, GetObjectsResult},
    proto,
};

pub struct PostgresConfig {
    pub uri: String,
}

pub struct PostgresConnector {
    pub config: PostgresConfig,
}

impl From<tokio_postgres::Error> for super::super::connector::ConnectorError {
    fn from(value: tokio_postgres::Error) -> Self {
        Self::PostgresError(value)
    }
}

#[async_trait::async_trait]
impl Connector for PostgresConnector {
    async fn get_objects(
        &mut self,
    ) -> Result<GetObjectsResult, crate::core::dbclient::connector::ConnectorError> {
        let (client, connection) = tokio_postgres::connect(&self.config.uri, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("PostgreSQL connection error: {error}");
            }
        });

        let database_name: String = client
            .query_one("SELECT current_database()", &[])
            .await?
            .get(0);
        let schemas = client
            .query(
                "
                SELECT schema_name
                FROM information_schema.schemata
                WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
                ORDER BY schema_name
            ",
                &[],
            )
            .await?;

        let tables = client
            .query(
                "
                SELECT table_schema, table_name
                FROM information_schema.tables
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
                  AND table_type = 'BASE TABLE'
                ORDER BY table_schema, table_name
            ",
                &[],
            )
            .await?;
        let views = client
            .query(
                "
                SELECT table_schema, table_name
                FROM information_schema.views
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
                ORDER BY table_schema, table_name
            ",
                &[],
            )
            .await?;
        let materialized_views = client
            .query(
                "
                SELECT schemaname, matviewname
                FROM pg_catalog.pg_matviews
                WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
                ORDER BY schemaname, matviewname
            ",
                &[],
            )
            .await?;
        let functions = client
            .query(
                "
                SELECT routine_schema, routine_name
                FROM information_schema.routines
                WHERE routine_schema NOT IN ('pg_catalog', 'information_schema')
                  AND routine_type = 'FUNCTION'
                ORDER BY routine_schema, routine_name
            ",
                &[],
            )
            .await?;

        let columns = client
            .query(
                "
                SELECT table_schema, table_name, column_name
                FROM information_schema.columns
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
                ORDER BY table_schema, table_name, ordinal_position
            ",
                &[],
            )
            .await?;
        let constraints = client
            .query(
                "
                SELECT table_schema, table_name, constraint_name
                FROM information_schema.table_constraints
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
                ORDER BY table_schema, table_name, constraint_name
            ",
                &[],
            )
            .await?;
        let indexes = client
            .query(
                "
                SELECT schemaname, tablename, indexname
                FROM pg_catalog.pg_indexes
                WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
                ORDER BY schemaname, tablename, indexname
            ",
                &[],
            )
            .await?;

        let mut table_names: HashMap<String, Vec<String>> = HashMap::new();
        let mut view_names: HashMap<String, Vec<String>> = HashMap::new();
        let mut materialized_view_names: HashMap<String, Vec<String>> = HashMap::new();
        let mut function_names: HashMap<String, Vec<String>> = HashMap::new();
        let mut table_columns: HashMap<(String, String), Vec<String>> = HashMap::new();
        let mut table_constraints: HashMap<(String, String), Vec<String>> = HashMap::new();
        let mut table_indexes: HashMap<(String, String), Vec<String>> = HashMap::new();

        for row in tables {
            table_names.entry(row.get(0)).or_default().push(row.get(1));
        }
        for row in views {
            view_names.entry(row.get(0)).or_default().push(row.get(1));
        }
        for row in materialized_views {
            materialized_view_names
                .entry(row.get(0))
                .or_default()
                .push(row.get(1));
        }
        for row in functions {
            function_names
                .entry(row.get(0))
                .or_default()
                .push(row.get(1));
        }
        for row in columns {
            table_columns
                .entry((row.get(0), row.get(1)))
                .or_default()
                .push(row.get(2));
        }
        for row in constraints {
            table_constraints
                .entry((row.get(0), row.get(1)))
                .or_default()
                .push(row.get(2));
        }
        for row in indexes {
            table_indexes
                .entry((row.get(0), row.get(1)))
                .or_default()
                .push(row.get(2));
        }

        let schemas = schemas
            .into_iter()
            .map(|row| {
                let schema_name: String = row.get(0);
                let schema_tables = table_names
                    .remove(&schema_name)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|table_name| {
                        let key = (schema_name.clone(), table_name.clone());
                        proto::common::PostgresTable {
                            descriptor: Some(proto::common::PostgresTableDescriptor {
                                name: table_name,
                            }),
                            columns: table_columns
                                .remove(&key)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|name| proto::common::PostgresTableColumn { name })
                                .collect(),
                            constrains: table_constraints
                                .remove(&key)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|name| proto::common::PostgresTableConstrain { name })
                                .collect(),
                            indexes: table_indexes
                                .remove(&key)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|name| proto::common::PostgresTableIndex { name })
                                .collect(),
                        }
                    })
                    .collect();

                proto::common::PostgresSchema {
                    descriptor: Some(proto::common::PostgresSchemaDescriptor {
                        name: schema_name.clone(),
                    }),
                    tables: schema_tables,
                    views: view_names
                        .remove(&schema_name)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| proto::common::PostgresView { name })
                        .collect(),
                    materialized_views: materialized_view_names
                        .remove(&schema_name)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| proto::common::PostgresView { name })
                        .collect(),
                    functions: function_names
                        .remove(&schema_name)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| proto::common::PostgresFunction { name })
                        .collect(),
                }
            })
            .collect();

        Ok(proto::common::DbObject {
            specification: Some(proto::common::db_object::Specification::Postgres(
                proto::common::PostgresObject {
                    object: Some(proto::common::postgres_object::Object::Database(
                        proto::common::PostgresDatabase {
                            descriptor: Some(proto::common::PostgresDatabaseDescriptor {
                                name: database_name,
                            }),
                            schemas,
                        },
                    )),
                },
            )),
        })
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::dbclient::connector::Connector;

    const POSTGRES_URI: &str = "postgresql://user:password@localhost/postgres?connect_timeout=10";

    fn test_connector() -> PostgresConnector {
        PostgresConnector {
            config: PostgresConfig {
                uri: POSTGRES_URI.to_string(),
            },
        }
    }

    async fn setup_schema(schema: &str) -> tokio_postgres::Client {
        let (client, connection) = tokio_postgres::connect(POSTGRES_URI, NoTls)
            .await
            .expect("failed to connect to the PostgreSQL test database");
        tokio::spawn(async move {
            connection.await.expect("PostgreSQL test connection failed");
        });
        client
            .batch_execute(&format!(
                "DROP SCHEMA IF EXISTS {schema} CASCADE; CREATE SCHEMA {schema};"
            ))
            .await
            .expect("failed to create the PostgreSQL test schema");
        client
    }

    #[ignore = "integration tests require the PostgreSQL service from tests.compose.yml"]
    #[tokio::test]
    async fn get_objects_returns_database_and_empty_schema() {
        let schema = "connector_impl_empty_schema";
        let mut client = setup_schema(schema).await;

        let mut connector = test_connector();
        let object = connector
            .get_objects()
            .await
            .expect("failed to fetch PostgreSQL objects");

        let database = match object.specification {
            Some(proto::common::db_object::Specification::Postgres(postgres)) => {
                match postgres.object {
                    Some(proto::common::postgres_object::Object::Database(database)) => database,
                    other => panic!("expected a database object, got {other:?}"),
                }
            }
            other => panic!("expected a PostgreSQL object, got {other:?}"),
        };

        assert_eq!(database.descriptor.unwrap().name, "postgres");
        let schema_object = database
            .schemas
            .iter()
            .find(|item| {
                item.descriptor
                    .as_ref()
                    .is_some_and(|descriptor| descriptor.name == schema)
            })
            .expect("test schema was not returned");
        assert!(schema_object.tables.is_empty());
        assert!(schema_object.views.is_empty());
        assert!(schema_object.materialized_views.is_empty());
        assert!(schema_object.functions.is_empty());

        client
            .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
            .await
            .expect("failed to clean up the PostgreSQL test schema");
    }

    #[ignore = "integration tests require the PostgreSQL service from tests.compose.yml"]
    #[tokio::test]
    async fn get_objects_returns_complete_schema_contents() {
        let schema = "connector_impl_full_schema";
        let mut client = setup_schema(schema).await;
        client
            .batch_execute(&format!(
                r#"
                CREATE TABLE {schema}.users (
                    id integer CONSTRAINT users_pk PRIMARY KEY,
                    name text NOT NULL,
                    age integer CONSTRAINT users_age_check CHECK (age >= 0)
                );
                CREATE INDEX users_name_idx ON {schema}.users (name);
                CREATE VIEW {schema}.active_users AS
                    SELECT id, name FROM {schema}.users WHERE age > 0;
                CREATE MATERIALIZED VIEW {schema}.users_snapshot AS
                    SELECT id, name FROM {schema}.users;
                CREATE FUNCTION {schema}.user_count()
                RETURNS integer
                LANGUAGE SQL
                IMMUTABLE
                AS $$ SELECT 0 $$;
                "#
            ))
            .await
            .expect("failed to create PostgreSQL test objects");

        let mut connector = test_connector();
        let object = connector
            .get_objects()
            .await
            .expect("failed to fetch PostgreSQL objects");
        let database = match object.specification {
            Some(proto::common::db_object::Specification::Postgres(postgres)) => {
                match postgres.object {
                    Some(proto::common::postgres_object::Object::Database(database)) => database,
                    other => panic!("expected a database object, got {other:?}"),
                }
            }
            other => panic!("expected a PostgreSQL object, got {other:?}"),
        };
        let schema_object = database
            .schemas
            .iter()
            .find(|item| {
                item.descriptor
                    .as_ref()
                    .is_some_and(|descriptor| descriptor.name == schema)
            })
            .expect("test schema was not returned");

        let table = schema_object
            .tables
            .iter()
            .find(|table| {
                table
                    .descriptor
                    .as_ref()
                    .is_some_and(|descriptor| descriptor.name == "users")
            })
            .expect("test table was not returned");
        let column_names: Vec<_> = table
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect();
        assert_eq!(column_names, ["id", "name", "age"]);
        assert!(table
            .constrains
            .iter()
            .any(|constraint| constraint.name == "users_pk"));
        assert!(table
            .constrains
            .iter()
            .any(|constraint| constraint.name == "users_age_check"));
        assert!(table
            .indexes
            .iter()
            .any(|index| index.name == "users_name_idx"));

        assert!(schema_object
            .views
            .iter()
            .any(|view| view.name == "active_users"));
        assert!(schema_object
            .materialized_views
            .iter()
            .any(|view| view.name == "users_snapshot"));
        assert!(schema_object
            .functions
            .iter()
            .any(|function| function.name == "user_count"));

        client
            .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
            .await
            .expect("failed to clean up the PostgreSQL test schema");
    }
}
