use std::collections::HashMap;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use tokio_postgres::NoTls;

use crate::core::{
    dbclient::connector::{
        AddRecordToObjectRequest, Connector, GetObjectsResult, ListAllItemsFromObjectRequest,
        ListAllItemsFromObjectResult, RemoveRecordFromObjectRequest, UpdateRecordOfObjectRequest,
    },
    proto::{
        self,
        common::{DbField, PostgresObjectDescriptor, PostgresRecord, PostgresTableDescriptor},
    },
};

pub struct PostgresConfig {
    pub uri: String,
}

pub struct PostgresConnector {
    pub config: PostgresConfig,
}

impl proto::common::DbField {
    fn from_pg_type(type_name: &str) -> Self {
        use proto::common::db_field::Field;

        let field = match type_name {
            "bool" => Some(Field::Boolean(proto::common::Boolean { boolean: None })),
            "int2" => Some(Field::I16(proto::common::Int16 { i16: None })),
            "int4" => Some(Field::I32(proto::common::Int32 { i32: None })),
            "int8" => Some(Field::I64(proto::common::Int64 { i64: None })),
            "varchar" | "text" => Some(Field::Str(proto::common::String { str: None })),
            "timestamptz" => Some(Field::Datetime(proto::common::Timestamp { datetime: None })),
            _ => None,
        };

        Self { field }
    }

    fn into_sql_value(&self) -> Option<String> {
        use proto::common::db_field::Field;

        match &self.field {
            Some(Field::Str(proto::common::String { str: Some(s) })) => Some(format!("'{s}'")),
            Some(Field::StrContainer(proto::common::StringContainer { strs })) => {
                let s = strs.join("\n");
                Some(format!("'{s}'"))
            }
            Some(Field::I8(proto::common::Int8 { i8: Some(i8) })) => Some(i8.to_string()),
            Some(Field::I16(proto::common::Int16 { i16: Some(i16) })) => Some(i16.to_string()),
            Some(Field::I32(proto::common::Int32 { i32: Some(i32) })) => Some(i32.to_string()),
            Some(Field::I64(proto::common::Int64 { i64: Some(i64) })) => Some(i64.to_string()),
            Some(Field::Boolean(proto::common::Boolean { boolean: Some(b) })) => {
                Some(b.to_string())
            }
            Some(Field::Datetime(proto::common::Timestamp {
                datetime: Some(timestamp),
            })) => {
                let s = timestamp.to_string();
                Some(format!("'{s}'"))
            }
            _ => None,
        }
    }
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
                SELECT table_schema, table_name, column_name, udt_name
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
        let mut table_columns: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
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
                .push((row.get(2), row.get(3)));
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
                                upstream_descriptor: Some(
                                    proto::common::PostgresSchemaDescriptor {
                                        upstream_descriptor: Some(
                                            proto::common::PostgresDatabaseDescriptor {
                                                name: database_name.clone(),
                                            },
                                        ),
                                        name: schema_name.clone(),
                                    },
                                ),
                                name: table_name,
                            }),
                            columns: table_columns
                                .remove(&key)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|(name, type_name)| proto::common::PostgresTableColumn {
                                    name,
                                    field: Some(DbField::from_pg_type(type_name.as_str())),
                                })
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
                        upstream_descriptor: Some(proto::common::PostgresDatabaseDescriptor {
                            name: database_name.clone(),
                        }),
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

    async fn get_object(
        &mut self,
        req: proto::common::DbObjectDescriptor,
    ) -> Result<proto::common::DbObject, crate::core::dbclient::connector::ConnectorError> {
        let descriptor = match req.descriptor {
            Some(proto::common::db_object_descriptor::Descriptor::Postgres(descriptor)) => {
                descriptor.descriptor.ok_or(
                    crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                        "empty PostgreSQL descriptor",
                    ),
                )?
            }
            None => {
                return Err(
                    crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                        "empty descriptor",
                    ),
                );
            }
        };

        let (client, connection) = tokio_postgres::connect(&self.config.uri, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("PostgreSQL connection error: {error}");
            }
        });

        let object = match descriptor {
            proto::common::postgres_object_descriptor::Descriptor::Database(descriptor) => {
                let current_database: String = client
                    .query_one("SELECT current_database()", &[])
                    .await?
                    .get(0);
                if current_database != descriptor.name {
                    return Err(
                        crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                            "database not found",
                        ),
                    );
                }

                load_database_object(&client, current_database).await?
            }
            proto::common::postgres_object_descriptor::Descriptor::Schema(descriptor) => {
                let database = descriptor.upstream_descriptor.as_ref().ok_or(
                    crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                        "schema database descriptor is required",
                    ),
                )?;
                let current_database: String = client
                    .query_one("SELECT current_database()", &[])
                    .await?
                    .get(0);
                if current_database != database.name {
                    return Err(
                        crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                            "database not found",
                        ),
                    );
                }
                let object =
                    load_schema_object(&client, database.name.clone(), descriptor.name).await?;
                proto::common::DbObject {
                    specification: Some(proto::common::db_object::Specification::Postgres(
                        proto::common::PostgresObject {
                            object: Some(proto::common::postgres_object::Object::Schema(object)),
                        },
                    )),
                }
            }
            proto::common::postgres_object_descriptor::Descriptor::Table(descriptor) => {
                let schema = descriptor.upstream_descriptor.ok_or(
                    crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                        "table schema descriptor is required",
                    ),
                )?;
                let database = schema.upstream_descriptor.ok_or(
                    crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                        "table database descriptor is required",
                    ),
                )?;
                let current_database: String = client
                    .query_one("SELECT current_database()", &[])
                    .await?
                    .get(0);
                if current_database != database.name {
                    return Err(
                        crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                            "database not found",
                        ),
                    );
                }
                let table =
                    load_table_object(&client, database.name, schema.name, descriptor.name).await?;
                proto::common::DbObject {
                    specification: Some(proto::common::db_object::Specification::Postgres(
                        proto::common::PostgresObject {
                            object: Some(proto::common::postgres_object::Object::Table(table)),
                        },
                    )),
                }
            }
        };

        Ok(object)
    }

    async fn list_all_items_from_object(
        &mut self,
        req: ListAllItemsFromObjectRequest,
    ) -> Result<ListAllItemsFromObjectResult, crate::core::dbclient::connector::ConnectorError>
    {
        let (client, connection) = tokio_postgres::connect(&self.config.uri, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("PostgreSQL connection error: {error}");
            }
        });

        if let Some(proto::common::db_object_descriptor::Descriptor::Postgres(
            PostgresObjectDescriptor {
                descriptor: Some(desc),
            },
        )) = req.descriptor
        {
            let table_name = match desc {
                proto::common::postgres_object_descriptor::Descriptor::Table(
                    postgres_table_descriptor,
                ) => postgres_table_descriptor.name,
                _ => {
                    return Err(
                        crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                            "only list table available",
                        ),
                    )
                }
            };

            let columns: Vec<(String, String)> = {
                let query = format!(
                    "
                    SELECT
                        column_name,
                        data_type
                    FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = '{}'
                    ORDER BY ordinal_position;
                    ",
                    table_name
                );
                let rows = client.query(&query, &[]).await?;
                let mut columns = vec![];
                for row in rows {
                    let column_name: String = row.get("column_name");
                    let data_type: String = row.get("data_type");
                    columns.push((column_name, data_type));
                }
                columns
            };

            let query = format!(
                "SELECT {} FROM {}",
                columns
                    .iter()
                    .map(|column| column.0.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                table_name
            );
            let rows = client.query(&query, &[]).await?;
            let mut proto_rows = vec![];
            for row in rows {
                let mut proto_row = proto::common::PostgresRecordTableRow { values: vec![] };
                let columns = row.columns();
                for column in columns {
                    let name = column.name();
                    let column_type_name = column.type_().name();
                    let value =
                        match column_type_name {
                            "bool" => {
                                let value: Option<bool> = row.try_get(name)?;
                                proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::Boolean(
                                        proto::common::Boolean { boolean: value },
                                    )),
                                }
                            }
                            "int2" => {
                                let value: Option<i16> = row.try_get(name)?;
                                proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::I16(
                                        proto::common::Int16 {
                                            i16: value.map(|v| v as i32),
                                        },
                                    )),
                                }
                            }
                            "int4" => {
                                let value: Option<i32> = row.try_get(name)?;
                                proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::I32(
                                        proto::common::Int32 { i32: value },
                                    )),
                                }
                            }
                            "int8" => {
                                let value: Option<i64> = row.try_get(name)?;
                                proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::I64(
                                        proto::common::Int64 { i64: value },
                                    )),
                                }
                            }
                            "varchar" => {
                                let value: Option<String> = row.try_get(name)?;
                                proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::Str(
                                        proto::common::String { str: value },
                                    )),
                                }
                            }
                            "timestamptz" => {
                                let value: std::time::SystemTime = row.try_get(name)?;
                                let datetime: DateTime<Utc> = value.into();
                                proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::Datetime(
                                        proto::common::Timestamp {
                                            datetime: Some(Timestamp {
                                                seconds: datetime.timestamp(),
                                                // TODO: also map nanos
                                                ..Default::default()
                                            }),
                                        },
                                    )),
                                }
                            }
                            _ => return Err(
                                crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                                    "[postgresql] failed to map value",
                                ),
                            ),
                        };
                    proto_row.values.push(value);
                }
                proto_rows.push(proto_row);
            }
            Ok(proto::common::DbRecord {
                specification: Some(proto::common::db_record::Specification::Postgres(
                    proto::common::PostgresRecord {
                        record: Some(proto::common::postgres_record::Record::Table(
                            proto::common::PostgresRecordTable {
                                columns: columns
                                    .iter()
                                    .map(|column| proto::common::PostgresTableColumn {
                                        name: column.0.clone(),
                                        field: Some(proto::common::DbField::from_pg_type(
                                            &column.1,
                                        )),
                                    })
                                    .collect(),
                                rows: proto_rows,
                            },
                        )),
                    },
                )),
            })
        } else {
            return Err(
                crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                    "empty descriptor",
                ),
            );
        }
    }

    async fn add_record_to_object(
        &mut self,
        req: AddRecordToObjectRequest,
    ) -> Result<(), crate::core::dbclient::connector::ConnectorError> {
        match (req.record.specification, req.descriptor.descriptor) {
            (
                Some(proto::common::db_record::Specification::Postgres(PostgresRecord {
                    record: Some(proto::common::postgres_record::Record::Table(table)),
                })),
                Some(proto::common::db_object_descriptor::Descriptor::Postgres(
                    PostgresObjectDescriptor {
                        descriptor:
                            Some(proto::common::postgres_object_descriptor::Descriptor::Table(
                                PostgresTableDescriptor {
                                    name: table_name, ..
                                },
                            )),
                    },
                )),
            ) => {
                //NOTE: on invalid table, whole operation should be rejected
                for row in &table.rows {
                    if table.columns.len() != row.values.len() {
                        return Err(
                            crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                                "columns and values should have equal size",
                            ),
                        );
                    }
                }

                let (mut client, connection) =
                    tokio_postgres::connect(&self.config.uri, NoTls).await?;
                tokio::spawn(async move {
                    if let Err(error) = connection.await {
                        eprintln!("PostgreSQL connection error: {error}");
                    }
                });

                let transaction = client.transaction().await?;
                for row in &table.rows {
                    let mut columns = vec![];
                    let mut fields = vec![];
                    for (col, field) in table.columns.iter().zip(row.values.iter()) {
                        let Some(rendered_field) = field.into_sql_value() else {
                            continue;
                        };
                        columns.push(col.name.clone());
                        fields.push(rendered_field);
                    }
                    if fields.is_empty() {
                        continue;
                    }
                    let columns = columns.join(",");
                    let fields = fields.join(",");

                    transaction
                        .execute(
                            &format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                &table_name, &columns, &fields
                            ),
                            &[],
                        )
                        .await?;
                }
                transaction.commit().await?;
                Ok(())
            }
            _ => Err(
                crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                    "passed record and descriptor couldn't be matched",
                ),
            ),
        }
    }

    async fn remove_record_from_object(
        &mut self,
        req: RemoveRecordFromObjectRequest,
    ) -> Result<(), crate::core::dbclient::connector::ConnectorError> {
        match (req.record.specification, req.descriptor.descriptor) {
            (
                Some(proto::common::db_record::Specification::Postgres(PostgresRecord {
                    record: Some(proto::common::postgres_record::Record::Table(table)),
                })),
                Some(proto::common::db_object_descriptor::Descriptor::Postgres(
                    PostgresObjectDescriptor {
                        descriptor:
                            Some(proto::common::postgres_object_descriptor::Descriptor::Table(
                                PostgresTableDescriptor {
                                    name: table_name, ..
                                },
                            )),
                    },
                )),
            ) => {
                //NOTE: on invalid table, whole operation should be rejected
                for row in &table.rows {
                    if table.columns.len() != row.values.len() {
                        return Err(
                            crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                                "columns and values should have equal size",
                            ),
                        );
                    }
                }

                let (mut client, connection) =
                    tokio_postgres::connect(&self.config.uri, NoTls).await?;
                tokio::spawn(async move {
                    if let Err(error) = connection.await {
                        eprintln!("PostgreSQL connection error: {error}");
                    }
                });

                let transaction = client.transaction().await?;
                for row in &table.rows {
                    let mut filters = vec![];
                    for (col, field) in table.columns.iter().zip(row.values.iter()) {
                        let Some(rendered_field) = field.into_sql_value() else {
                            continue;
                        };
                        filters.push(format!("{}={}", col.name.clone(), rendered_field));
                    }
                    if filters.is_empty() {
                        continue;
                    }
                    let filters = filters.join(" AND ");
                    transaction
                        .execute(
                            &format!("DELETE FROM {} WHERE {}", &table_name, &filters),
                            &[],
                        )
                        .await?;
                }
                transaction.commit().await?;
                Ok(())
            }
            _ => Err(
                crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                    "passed record and descriptor couldn't be matched",
                ),
            ),
        }
    }

    async fn update_record_from_object(
        &mut self,
        req: UpdateRecordOfObjectRequest,
    ) -> Result<(), crate::core::dbclient::connector::ConnectorError> {
        match (
            req.old_object.specification,
            req.new_object.specification,
            req.descriptor.descriptor,
        ) {
            (
                Some(proto::common::db_record::Specification::Postgres(PostgresRecord {
                    record: Some(proto::common::postgres_record::Record::Table(old_table)),
                })),
                Some(proto::common::db_record::Specification::Postgres(PostgresRecord {
                    record: Some(proto::common::postgres_record::Record::Table(new_table)),
                })),
                Some(proto::common::db_object_descriptor::Descriptor::Postgres(
                    PostgresObjectDescriptor {
                        descriptor:
                            Some(proto::common::postgres_object_descriptor::Descriptor::Table(
                                PostgresTableDescriptor {
                                    name: table_name, ..
                                },
                            )),
                    },
                )),
            ) => {
                //NOTE: on invalid table, whole operation should be rejected
                for row in &old_table.rows {
                    if old_table.columns.len() != row.values.len() {
                        return Err(
                            crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                                "columns and values should have equal size",
                            ),
                        );
                    }
                }
                for row in &new_table.rows {
                    if new_table.columns.len() != row.values.len() {
                        return Err(
                            crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                                "columns and values should have equal size",
                            ),
                        );
                    }
                }

                let (mut client, connection) =
                    tokio_postgres::connect(&self.config.uri, NoTls).await?;
                tokio::spawn(async move {
                    if let Err(error) = connection.await {
                        eprintln!("PostgreSQL connection error: {error}");
                    }
                });

                let transaction = client.transaction().await?;

                for (old_row, new_row) in old_table.rows.iter().zip(new_table.rows) {
                    let mut filter = vec![];
                    for (col, field) in old_table.columns.iter().zip(old_row.values.iter()) {
                        let Some(rendered_field) = field.into_sql_value() else {
                            continue;
                        };
                        filter.push(format!("{}={}", col.name.clone(), rendered_field));
                    }
                    if filter.is_empty() {
                        continue;
                    }
                    let filter = filter.join(" AND ");

                    let mut replaces = vec![];
                    for (col, field) in new_table.columns.iter().zip(new_row.values.iter()) {
                        let Some(rendered_field) = field.into_sql_value() else {
                            continue;
                        };
                        replaces.push(format!("{}={}", col.name.clone(), rendered_field));
                    }
                    if replaces.is_empty() {
                        continue;
                    }
                    let replaces = replaces.join(",");

                    transaction
                        .execute(
                            &format!("UPDATE {table_name} SET {replaces} WHERE {filter}"),
                            &[],
                        )
                        .await?;
                }
                transaction.commit().await?;
                Ok(())
            }
            _ => Err(
                crate::core::dbclient::connector::ConnectorError::InvalidRequest(
                    "passed record and descriptor couldn't be matched",
                ),
            ),
        }
    }
}

async fn load_database_object(
    client: &tokio_postgres::Client,
    database_name: String,
) -> Result<proto::common::DbObject, crate::core::dbclient::connector::ConnectorError> {
    let schemas = client
        .query(
            "SELECT schema_name
             FROM information_schema.schemata
             WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
             ORDER BY schema_name",
            &[],
        )
        .await?;

    let mut schema_objects = Vec::with_capacity(schemas.len());
    for row in schemas {
        schema_objects.push(load_schema_object(client, database_name.clone(), row.get(0)).await?);
    }

    Ok(proto::common::DbObject {
        specification: Some(proto::common::db_object::Specification::Postgres(
            proto::common::PostgresObject {
                object: Some(proto::common::postgres_object::Object::Database(
                    proto::common::PostgresDatabase {
                        descriptor: Some(proto::common::PostgresDatabaseDescriptor {
                            name: database_name.clone(),
                        }),
                        schemas: schema_objects,
                    },
                )),
            },
        )),
    })
}

async fn load_schema_object(
    client: &tokio_postgres::Client,
    database_name: String,
    schema_name: String,
) -> Result<proto::common::PostgresSchema, crate::core::dbclient::connector::ConnectorError> {
    let schema_exists = client
        .query_opt(
            "SELECT 1
             FROM information_schema.schemata
             WHERE schema_name = $1",
            &[&schema_name],
        )
        .await?
        .is_some();
    if !schema_exists {
        return Err(
            crate::core::dbclient::connector::ConnectorError::InvalidRequest("schema not found"),
        );
    }

    let tables = client
        .query(
            "SELECT table_name
             FROM information_schema.tables
             WHERE table_schema = $1 AND table_type = 'BASE TABLE'
             ORDER BY table_name",
            &[&schema_name],
        )
        .await?;
    let mut table_objects = Vec::with_capacity(tables.len());
    for row in tables {
        table_objects.push(
            load_table_object(
                client,
                database_name.clone(),
                schema_name.clone(),
                row.get(0),
            )
            .await?,
        );
    }

    let views = client
        .query(
            "SELECT table_name
             FROM information_schema.views
             WHERE table_schema = $1
             ORDER BY table_name",
            &[&schema_name],
        )
        .await?
        .into_iter()
        .map(|row| proto::common::PostgresView { name: row.get(0) })
        .collect();
    let materialized_views = client
        .query(
            "SELECT matviewname
             FROM pg_catalog.pg_matviews
             WHERE schemaname = $1
             ORDER BY matviewname",
            &[&schema_name],
        )
        .await?
        .into_iter()
        .map(|row| proto::common::PostgresView { name: row.get(0) })
        .collect();
    let functions = client
        .query(
            "SELECT routine_name
             FROM information_schema.routines
             WHERE routine_schema = $1 AND routine_type = 'FUNCTION'
             ORDER BY routine_name",
            &[&schema_name],
        )
        .await?
        .into_iter()
        .map(|row| proto::common::PostgresFunction { name: row.get(0) })
        .collect();

    Ok(proto::common::PostgresSchema {
        descriptor: Some(proto::common::PostgresSchemaDescriptor {
            upstream_descriptor: Some(proto::common::PostgresDatabaseDescriptor {
                name: database_name,
            }),
            name: schema_name,
        }),
        tables: table_objects,
        views,
        materialized_views,
        functions,
    })
}

async fn load_table_object(
    client: &tokio_postgres::Client,
    database_name: String,
    schema_name: String,
    table_name: String,
) -> Result<proto::common::PostgresTable, crate::core::dbclient::connector::ConnectorError> {
    let table_exists = client
        .query_opt(
            "SELECT 1
             FROM information_schema.tables
             WHERE table_schema = $1 AND table_name = $2 AND table_type = 'BASE TABLE'",
            &[&schema_name, &table_name],
        )
        .await?
        .is_some();
    if !table_exists {
        return Err(
            crate::core::dbclient::connector::ConnectorError::InvalidRequest("table not found"),
        );
    }

    let columns = client
        .query(
            "SELECT column_name, udt_name
             FROM information_schema.columns
             WHERE table_schema = $1 AND table_name = $2
             ORDER BY ordinal_position",
            &[&schema_name, &table_name],
        )
        .await?
        .into_iter()
        .map(|row| {
            let type_name: String = row.get(1);
            proto::common::PostgresTableColumn {
                name: row.get(0),
                field: Some(proto::common::DbField::from_pg_type(type_name.as_str())),
            }
        })
        .collect();
    let constrains = client
        .query(
            "SELECT constraint_name
             FROM information_schema.table_constraints
             WHERE table_schema = $1 AND table_name = $2
             ORDER BY constraint_name",
            &[&schema_name, &table_name],
        )
        .await?
        .into_iter()
        .map(|row| proto::common::PostgresTableConstrain { name: row.get(0) })
        .collect();
    let indexes = client
        .query(
            "SELECT indexname
             FROM pg_catalog.pg_indexes
             WHERE schemaname = $1 AND tablename = $2
             ORDER BY indexname",
            &[&schema_name, &table_name],
        )
        .await?
        .into_iter()
        .map(|row| proto::common::PostgresTableIndex { name: row.get(0) })
        .collect();

    Ok(proto::common::PostgresTable {
        descriptor: Some(proto::common::PostgresTableDescriptor {
            upstream_descriptor: Some(proto::common::PostgresSchemaDescriptor {
                upstream_descriptor: Some(proto::common::PostgresDatabaseDescriptor {
                    name: database_name,
                }),
                name: schema_name,
            }),
            name: table_name,
        }),
        columns,
        constrains,
        indexes,
    })
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
