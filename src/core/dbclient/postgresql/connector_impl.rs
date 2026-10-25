use std::collections::HashMap;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use tokio_postgres::{NoTls, Row};

use crate::core::{
    dbclient::connector::{
        AddRecordToObjectRequest, Connector, GetObjectsResult, ListAllItemsFromObjectRequest,
        ListAllItemsFromObjectResult, RemoveRecordFromObjectRequest, UpdateRecordOfObjectRequest,
    },
    proto::{
        self,
        postgresql::{
            postgres_field::Field, PostgresObjectDescriptor, PostgresRecord,
            PostgresTableDescriptor,
        },
    },
};

pub struct PostgresConfig {
    pub uri: String,
}

pub struct PostgresConnector {
    pub config: PostgresConfig,
}

impl proto::postgresql::PostgresField {
    pub fn into_pg_type(&self) -> Option<&'static str> {
        match &self.field {
            Some(proto::postgresql::postgres_field::Field::Unsupported(_)) => Some("unsupported"),
            Some(proto::postgresql::postgres_field::Field::Bigint(_)) => Some("bigint"),
            Some(proto::postgresql::postgres_field::Field::Bigserial(_)) => Some("bigserial"),
            Some(proto::postgresql::postgres_field::Field::DoublePrecision(_)) => {
                Some("double precision")
            }
            Some(proto::postgresql::postgres_field::Field::Integer(_)) => Some("integer"),
            Some(proto::postgresql::postgres_field::Field::Numeric(_)) => Some("numeric"),
            Some(proto::postgresql::postgres_field::Field::Real(_)) => Some("real"),
            Some(proto::postgresql::postgres_field::Field::Smallint(_)) => Some("smallint"),
            Some(proto::postgresql::postgres_field::Field::Smallserial(_)) => Some("smallserial"),
            Some(proto::postgresql::postgres_field::Field::Serial(_)) => Some("serial"),
            Some(proto::postgresql::postgres_field::Field::Bit(_)) => Some("bit"),
            Some(proto::postgresql::postgres_field::Field::Varbit(_)) => Some("varbit"),
            Some(proto::postgresql::postgres_field::Field::Char(_)) => Some("char"),
            Some(proto::postgresql::postgres_field::Field::Varchar(_)) => Some("varchar"),
            Some(proto::postgresql::postgres_field::Field::Text(_)) => Some("text"),
            Some(proto::postgresql::postgres_field::Field::Bytea(_)) => Some("bytea"),
            Some(proto::postgresql::postgres_field::Field::Date(_)) => Some("date"),
            Some(proto::postgresql::postgres_field::Field::Time(_)) => Some("time"),
            Some(proto::postgresql::postgres_field::Field::Timetz(_)) => Some("timetz"),
            Some(proto::postgresql::postgres_field::Field::Timestamp(_)) => Some("timestamp"),
            Some(proto::postgresql::postgres_field::Field::Timestamptz(_)) => Some("timestamptz"),
            Some(proto::postgresql::postgres_field::Field::Interval(_)) => Some("interval"),
            Some(proto::postgresql::postgres_field::Field::Boolean(_)) => Some("boolean"),
            Some(proto::postgresql::postgres_field::Field::Money(_)) => Some("money"),
            Some(proto::postgresql::postgres_field::Field::Cidr(_)) => Some("cidr"),
            Some(proto::postgresql::postgres_field::Field::Inet(_)) => Some("inet"),
            Some(proto::postgresql::postgres_field::Field::Box(_)) => Some("box"),
            Some(proto::postgresql::postgres_field::Field::Circle(_)) => Some("circle"),
            Some(proto::postgresql::postgres_field::Field::Line(_)) => Some("line"),
            Some(proto::postgresql::postgres_field::Field::Lseg(_)) => Some("lseg"),
            Some(proto::postgresql::postgres_field::Field::Path(_)) => Some("path"),
            Some(proto::postgresql::postgres_field::Field::Point(_)) => Some("point"),
            Some(proto::postgresql::postgres_field::Field::Polygon(_)) => Some("polygon"),
            Some(proto::postgresql::postgres_field::Field::Json(_)) => Some("json"),
            Some(proto::postgresql::postgres_field::Field::Jsonb(_)) => Some("jsonb"),
            Some(proto::postgresql::postgres_field::Field::Tsquery(_)) => Some("tsquery"),
            Some(proto::postgresql::postgres_field::Field::Tsvector(_)) => Some("tsvector"),
            Some(proto::postgresql::postgres_field::Field::Uuid(_)) => Some("uuid"),
            Some(proto::postgresql::postgres_field::Field::Xml(_)) => Some("xml"),
            None => None,
        }
    }

    fn from_pg_type(type_name: &str, row: Option<&Row>, column_name: Option<&str>) -> Self {
        macro_rules! value {
            ($type:ty) => {
                match (row, column_name) {
                    (Some(row), Some(column_name)) => {
                        row.try_get::<_, Option<$type>>(column_name).ok().flatten()
                    }
                    _ => None,
                }
            };
        }

        let field = match type_name {
            "bigint" | "int8" => Some(Field::Bigint(proto::postgresql::Bigint {
                bigint: value!(i64),
            })),
            "bigserial" | "serial8" => Some(Field::Bigserial(proto::postgresql::Bigserial {
                bigserial: value!(i64),
            })),
            "double precision" | "float8" => {
                Some(Field::DoublePrecision(proto::postgresql::DoublePrecision {
                    double_precision: value!(f64),
                }))
            }
            "integer" | "int4" | "int" => Some(Field::Integer(proto::postgresql::Integer {
                integer: value!(i32),
            })),
            "numeric" | "decimal" => Some(Field::Numeric(proto::postgresql::Numeric {
                numeric: value!(String),
            })),
            "real" | "float4" => Some(Field::Real(proto::postgresql::Real { real: value!(f32) })),
            "smallint" | "int2" => Some(Field::Smallint(proto::postgresql::Smallint {
                smallint: value!(i16).map(i32::from),
            })),
            "smallserial" | "serial2" => Some(Field::Smallserial(proto::postgresql::Smallserial {
                smallserial: value!(i32),
            })),
            "serial" | "serial4" => Some(Field::Serial(proto::postgresql::Serial {
                serial: value!(i32),
            })),
            "bit" => Some(Field::Bit(proto::postgresql::Bit {
                bit: value!(String),
            })),
            "varbit" => Some(Field::Varbit(proto::postgresql::Varbit {
                varbit: value!(String),
            })),
            "char" | "character" | "bpchar" => Some(Field::Char(proto::postgresql::Char {
                r#char: value!(String),
            })),
            "varchar" | "character varying" => Some(Field::Varchar(proto::postgresql::Varchar {
                varchar: value!(String),
            })),
            "text" => Some(Field::Text(proto::postgresql::Text {
                text: value!(String),
            })),
            "bytea" => Some(Field::Bytea(proto::postgresql::Bytea {
                bytea: value!(Vec<u8>),
            })),
            "date" => Some(Field::Date(proto::postgresql::Date {
                date: value!(String),
            })),
            "time" | "time without time zone" => Some(Field::Time(proto::postgresql::Time {
                time: value!(String),
            })),
            "timetz" | "time with time zone" => Some(Field::Timetz(proto::postgresql::Timetz {
                timetz: value!(String),
            })),
            "timestamp" | "timestamp without time zone" => {
                Some(Field::Timestamp(proto::postgresql::Timestamp {
                    datetime: match value!(String) {
                        Some(value) => DateTime::parse_from_str(
                            &format!("{value} UTC"),
                            "%Y-%m-%d %H:%M:%S%.f %Z",
                        )
                        .ok()
                        .map(|value| Timestamp {
                            seconds: value.timestamp(),
                            nanos: value.timestamp_subsec_nanos() as i32,
                        }),
                        None => None,
                    },
                }))
            }
            "timestamptz" | "timestamp with time zone" => {
                Some(Field::Timestamptz(proto::postgresql::Timestamptz {
                    timestamptz: value!(std::time::SystemTime).map(|value| {
                        let value: DateTime<Utc> = value.into();
                        Timestamp {
                            seconds: value.timestamp(),
                            nanos: value.timestamp_subsec_nanos() as i32,
                        }
                    }),
                }))
            }
            "interval" => Some(Field::Interval(proto::postgresql::Interval {
                interval: value!(String),
            })),
            "bool" | "boolean" => Some(Field::Boolean(proto::postgresql::Boolean {
                boolean: value!(bool),
            })),
            "money" => Some(Field::Money(proto::postgresql::Money {
                money: value!(String),
            })),
            "cidr" => Some(Field::Cidr(proto::postgresql::Cidr {
                cidr: value!(String),
            })),
            "inet" => Some(Field::Inet(proto::postgresql::Inet {
                inet: value!(String),
            })),
            "box" => Some(Field::Box(proto::postgresql::Box {
                r#box: value!(String),
            })),
            "circle" => Some(Field::Circle(proto::postgresql::Circle {
                circle: value!(String),
            })),
            "line" => Some(Field::Line(proto::postgresql::Line {
                line: value!(String),
            })),
            "lseg" => Some(Field::Lseg(proto::postgresql::Lseg {
                lseg: value!(String),
            })),
            "path" => Some(Field::Path(proto::postgresql::Path {
                path: value!(String),
            })),
            "point" => Some(Field::Point(proto::postgresql::Point {
                point: value!(String),
            })),
            "polygon" => Some(Field::Polygon(proto::postgresql::Polygon {
                polygon: value!(String),
            })),
            "json" => Some(Field::Json(proto::postgresql::Json {
                json: value!(String),
            })),
            "jsonb" => Some(Field::Jsonb(proto::postgresql::Jsonb {
                jsonb: value!(String),
            })),
            "tsquery" => Some(Field::Tsquery(proto::postgresql::Tsquery {
                tsquery: value!(String),
            })),
            "tsvector" => Some(Field::Tsvector(proto::postgresql::Tsvector {
                tsvector: value!(String),
            })),
            "uuid" => Some(Field::Uuid(proto::postgresql::Uuid {
                uuid: value!(String),
            })),
            "xml" => Some(Field::Xml(proto::postgresql::Xml {
                xml: value!(String),
            })),
            _ => Some(Field::Unsupported(proto::postgresql::Unsupported {
                value: value!(String).or_else(|| Some(type_name.to_owned())),
            })),
        };

        Self { field }
    }

    fn into_sql_value(&self) -> Option<String> {
        fn quote(value: &str) -> String {
            format!("'{}'", value.replace('\'', "''"))
        }

        fn quote_timestamp(timestamp: &Timestamp) -> Option<String> {
            DateTime::<Utc>::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
                .map(|datetime| quote(&datetime.to_rfc3339()))
        }

        match &self.field {
            Some(Field::Unsupported(value)) => value.value.as_deref().map(quote),
            Some(Field::Bigint(value)) => value.bigint.map(|value| value.to_string()),
            Some(Field::Bigserial(value)) => value.bigserial.map(|value| value.to_string()),
            Some(Field::DoublePrecision(value)) => {
                value.double_precision.map(|value| value.to_string())
            }
            Some(Field::Integer(value)) => value.integer.map(|value| value.to_string()),
            Some(Field::Numeric(value)) => value.numeric.as_deref().map(str::to_owned),
            Some(Field::Real(value)) => value.real.map(|value| value.to_string()),
            Some(Field::Smallint(value)) => value.smallint.map(|value| value.to_string()),
            Some(Field::Smallserial(value)) => value.smallserial.map(|value| value.to_string()),
            Some(Field::Serial(value)) => value.serial.map(|value| value.to_string()),

            Some(Field::Bit(value)) => value.bit.as_deref().map(quote),
            Some(Field::Varbit(value)) => value.varbit.as_deref().map(quote),
            Some(Field::Char(value)) => value.r#char.as_deref().map(quote),
            Some(Field::Varchar(value)) => value.varchar.as_deref().map(quote),
            Some(Field::Text(value)) => value.text.as_deref().map(quote),
            Some(Field::Bytea(value)) => value.bytea.as_deref().map(|bytes| {
                let hex = bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                quote(&format!("\\x{hex}"))
            }),

            Some(Field::Date(value)) => value.date.as_deref().map(quote),
            Some(Field::Time(value)) => value.time.as_deref().map(quote),
            Some(Field::Timetz(value)) => value.timetz.as_deref().map(quote),
            Some(Field::Timestamp(value)) => value.datetime.as_ref().and_then(quote_timestamp),
            Some(Field::Timestamptz(value)) => value.timestamptz.as_ref().and_then(quote_timestamp),
            Some(Field::Interval(value)) => value.interval.as_deref().map(quote),

            Some(Field::Boolean(value)) => value.boolean.map(|value| value.to_string()),
            Some(Field::Money(value)) => value.money.as_deref().map(quote),
            Some(Field::Cidr(value)) => value.cidr.as_deref().map(quote),
            Some(Field::Inet(value)) => value.inet.as_deref().map(quote),
            Some(Field::Box(value)) => value.r#box.as_deref().map(quote),
            Some(Field::Circle(value)) => value.circle.as_deref().map(quote),
            Some(Field::Line(value)) => value.line.as_deref().map(quote),
            Some(Field::Lseg(value)) => value.lseg.as_deref().map(quote),
            Some(Field::Path(value)) => value.path.as_deref().map(quote),
            Some(Field::Point(value)) => value.point.as_deref().map(quote),
            Some(Field::Polygon(value)) => value.polygon.as_deref().map(quote),
            Some(Field::Json(value)) => value.json.as_deref().map(quote),
            Some(Field::Jsonb(value)) => value.jsonb.as_deref().map(quote),
            Some(Field::Tsquery(value)) => value.tsquery.as_deref().map(quote),
            Some(Field::Tsvector(value)) => value.tsvector.as_deref().map(quote),
            Some(Field::Uuid(value)) => value.uuid.as_deref().map(quote),
            Some(Field::Xml(value)) => value.xml.as_deref().map(quote),
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
                        proto::postgresql::PostgresTable {
                            descriptor: Some(proto::postgresql::PostgresTableDescriptor {
                                upstream_descriptor: Some(
                                    proto::postgresql::PostgresSchemaDescriptor {
                                        upstream_descriptor: Some(
                                            proto::postgresql::PostgresDatabaseDescriptor {
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
                                .map(|(name, type_name)| proto::postgresql::PostgresTableColumn {
                                    name,
                                    field: Some(proto::postgresql::PostgresField::from_pg_type(
                                        type_name.as_str(),
                                        None,
                                        None,
                                    )),
                                })
                                .collect(),
                            constrains: table_constraints
                                .remove(&key)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|name| proto::postgresql::PostgresTableConstrain { name })
                                .collect(),
                            indexes: table_indexes
                                .remove(&key)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|name| proto::postgresql::PostgresTableIndex { name })
                                .collect(),
                        }
                    })
                    .collect();

                proto::postgresql::PostgresSchema {
                    descriptor: Some(proto::postgresql::PostgresSchemaDescriptor {
                        upstream_descriptor: Some(proto::postgresql::PostgresDatabaseDescriptor {
                            name: database_name.clone(),
                        }),
                        name: schema_name.clone(),
                    }),
                    tables: schema_tables,
                    views: view_names
                        .remove(&schema_name)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| proto::postgresql::PostgresView { name })
                        .collect(),
                    materialized_views: materialized_view_names
                        .remove(&schema_name)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| proto::postgresql::PostgresView { name })
                        .collect(),
                    functions: function_names
                        .remove(&schema_name)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|name| proto::postgresql::PostgresFunction { name })
                        .collect(),
                }
            })
            .collect();

        Ok(proto::common::DbObject {
            specification: Some(proto::common::db_object::Specification::Postgres(
                proto::postgresql::PostgresObject {
                    object: Some(proto::postgresql::postgres_object::Object::Database(
                        proto::postgresql::PostgresDatabase {
                            descriptor: Some(proto::postgresql::PostgresDatabaseDescriptor {
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
            proto::postgresql::postgres_object_descriptor::Descriptor::Database(descriptor) => {
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
            proto::postgresql::postgres_object_descriptor::Descriptor::Schema(descriptor) => {
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
                        proto::postgresql::PostgresObject {
                            object: Some(proto::postgresql::postgres_object::Object::Schema(
                                object,
                            )),
                        },
                    )),
                }
            }
            proto::postgresql::postgres_object_descriptor::Descriptor::Table(descriptor) => {
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
                        proto::postgresql::PostgresObject {
                            object: Some(proto::postgresql::postgres_object::Object::Table(table)),
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
            proto::postgresql::PostgresObjectDescriptor {
                descriptor: Some(desc),
            },
        )) = req.descriptor
        {
            let table_name = match desc {
                proto::postgresql::postgres_object_descriptor::Descriptor::Table(
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
                let mut proto_row = proto::postgresql::PostgresRecordTableRow { values: vec![] };
                let columns = row.columns();
                for column in columns {
                    let name = column.name();
                    let column_type_name = column.type_().name();
                    let value = proto::postgresql::PostgresField::from_pg_type(
                        column_type_name,
                        Some(&row),
                        Some(name),
                    );
                    proto_row.values.push(value);
                }
                proto_rows.push(proto_row);
            }
            Ok(proto::common::DbRecord {
                specification: Some(proto::common::db_record::Specification::Postgres(
                    proto::postgresql::PostgresRecord {
                        record: Some(proto::postgresql::postgres_record::Record::Table(
                            proto::postgresql::PostgresRecordTable {
                                columns: columns
                                    .iter()
                                    .map(|column| proto::postgresql::PostgresTableColumn {
                                        name: column.0.clone(),
                                        field: Some(
                                            proto::postgresql::PostgresField::from_pg_type(
                                                &column.1, None, None,
                                            ),
                                        ),
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
                    record: Some(proto::postgresql::postgres_record::Record::Table(table)),
                })),
                Some(proto::common::db_object_descriptor::Descriptor::Postgres(
                    PostgresObjectDescriptor {
                        descriptor:
                            Some(proto::postgresql::postgres_object_descriptor::Descriptor::Table(
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
                    record: Some(proto::postgresql::postgres_record::Record::Table(table)),
                })),
                Some(proto::common::db_object_descriptor::Descriptor::Postgres(
                    PostgresObjectDescriptor {
                        descriptor:
                            Some(proto::postgresql::postgres_object_descriptor::Descriptor::Table(
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
                    record: Some(proto::postgresql::postgres_record::Record::Table(old_table)),
                })),
                Some(proto::common::db_record::Specification::Postgres(PostgresRecord {
                    record: Some(proto::postgresql::postgres_record::Record::Table(new_table)),
                })),
                Some(proto::common::db_object_descriptor::Descriptor::Postgres(
                    PostgresObjectDescriptor {
                        descriptor:
                            Some(proto::postgresql::postgres_object_descriptor::Descriptor::Table(
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
            proto::postgresql::PostgresObject {
                object: Some(proto::postgresql::postgres_object::Object::Database(
                    proto::postgresql::PostgresDatabase {
                        descriptor: Some(proto::postgresql::PostgresDatabaseDescriptor {
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
) -> Result<proto::postgresql::PostgresSchema, crate::core::dbclient::connector::ConnectorError> {
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
        .map(|row| proto::postgresql::PostgresView { name: row.get(0) })
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
        .map(|row| proto::postgresql::PostgresView { name: row.get(0) })
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
        .map(|row| proto::postgresql::PostgresFunction { name: row.get(0) })
        .collect();

    Ok(proto::postgresql::PostgresSchema {
        descriptor: Some(proto::postgresql::PostgresSchemaDescriptor {
            upstream_descriptor: Some(proto::postgresql::PostgresDatabaseDescriptor {
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
) -> Result<proto::postgresql::PostgresTable, crate::core::dbclient::connector::ConnectorError> {
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
            proto::postgresql::PostgresTableColumn {
                name: row.get(0),
                field: Some(proto::postgresql::PostgresField::from_pg_type(
                    type_name.as_str(),
                    None,
                    None,
                )),
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
        .map(|row| proto::postgresql::PostgresTableConstrain { name: row.get(0) })
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
        .map(|row| proto::postgresql::PostgresTableIndex { name: row.get(0) })
        .collect();

    Ok(proto::postgresql::PostgresTable {
        descriptor: Some(proto::postgresql::PostgresTableDescriptor {
            upstream_descriptor: Some(proto::postgresql::PostgresSchemaDescriptor {
                upstream_descriptor: Some(proto::postgresql::PostgresDatabaseDescriptor {
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
        let client = setup_schema(schema).await;

        let mut connector = test_connector();
        let object = connector
            .get_objects()
            .await
            .expect("failed to fetch PostgreSQL objects");

        let database = match object.specification {
            Some(proto::common::db_object::Specification::Postgres(postgres)) => {
                match postgres.object {
                    Some(proto::postgresql::postgres_object::Object::Database(database)) => {
                        database
                    }
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
        let client = setup_schema(schema).await;
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
                    Some(proto::postgresql::postgres_object::Object::Database(database)) => {
                        database
                    }
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
