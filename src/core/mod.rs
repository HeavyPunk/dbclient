mod dbclient;
pub mod server;

pub mod proto {
    pub mod connections {
        include!(concat!(env!("OUT_DIR"), "/dbclient.connections.rs"));
    }
    pub mod objects {
        include!(concat!(env!("OUT_DIR"), "/dbclient.objects.rs"));
    }
    pub mod queries {
        include!(concat!(env!("OUT_DIR"), "/dbclient.queries.rs"));
    }
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/dbclient.common.rs"));
    }
    pub mod postgresql {
        include!(concat!(env!("OUT_DIR"), "/dbclient.postgresql.rs"));
    }
    pub mod redis {
        include!(concat!(env!("OUT_DIR"), "/dbclient.redis.rs"));
    }

    pub mod impls {
        use std::{fmt::Display, str::FromStr};

        use crate::core::proto;

        fn bytes_to_hex(bytes: &[u8]) -> String {
            bytes.iter().map(|byte| format!("{byte:02x}")).collect()
        }

        fn parse_optional<T>(input: &str) -> anyhow::Result<Option<T>>
        where
            T: FromStr,
            T::Err: Display,
        {
            if input.is_empty() {
                Ok(None)
            } else {
                input
                    .parse::<T>()
                    .map(Some)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))
            }
        }

        fn postgres_field(
            field: proto::postgresql::postgres_field::Field,
        ) -> proto::common::db_field::Field {
            proto::common::db_field::Field::Postgres(proto::postgresql::PostgresField {
                field: Some(field),
            })
        }

        fn optional_string(input: &str) -> Option<String> {
            (!input.is_empty()).then(|| input.to_owned())
        }

        fn parse_hex(input: &str) -> anyhow::Result<Option<Vec<u8>>> {
            if input.is_empty() {
                return Ok(None);
            }

            let input = input.strip_prefix("\\x").unwrap_or(input);
            if input.len() % 2 != 0 {
                anyhow::bail!("hex value must contain an even number of digits");
            }

            input
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| {
                    std::str::from_utf8(pair)
                        .map_err(|error| anyhow::anyhow!(error.to_string()))
                        .and_then(|pair| {
                            u8::from_str_radix(pair, 16)
                                .map_err(|error| anyhow::anyhow!(error.to_string()))
                        })
                })
                .collect::<anyhow::Result<Vec<_>>>()
                .map(Some)
        }

        fn parse_timestamp(input: &str) -> anyhow::Result<Option<prost_types::Timestamp>> {
            if input.is_empty() {
                return Ok(None);
            }

            let timestamp = chrono::DateTime::parse_from_rfc3339(input)?;
            Ok(Some(prost_types::Timestamp {
                seconds: timestamp.timestamp(),
                nanos: timestamp.timestamp_subsec_nanos() as i32,
            }))
        }

        impl Into<Option<String>> for proto::common::db_field::Field {
            fn into(self) -> Option<String> {
                match self {
                    proto::common::db_field::Field::Postgres(
                        proto::postgresql::PostgresField { field: Some(field) },
                    ) => match field {
                        proto::postgresql::postgres_field::Field::Unsupported(unsupported) => {
                            unsupported.value
                        }

                        proto::postgresql::postgres_field::Field::Bigint(bigint) => {
                            bigint.bigint.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Bigserial(bigserial) => {
                            bigserial.bigserial.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::DoublePrecision(
                            double_precision,
                        ) => double_precision.double_precision.map(|v| v.to_string()),
                        proto::postgresql::postgres_field::Field::Integer(integer) => {
                            integer.integer.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Numeric(numeric) => {
                            numeric.numeric
                        }
                        proto::postgresql::postgres_field::Field::Real(real) => {
                            real.real.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Smallint(smallint) => {
                            smallint.smallint.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Smallserial(smallserial) => {
                            smallserial.smallserial.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Serial(serial) => {
                            serial.serial.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Bit(bit) => bit.bit,
                        proto::postgresql::postgres_field::Field::Varbit(varbit) => varbit.varbit,
                        proto::postgresql::postgres_field::Field::Char(char) => char.r#char,
                        proto::postgresql::postgres_field::Field::Varchar(varchar) => {
                            varchar.varchar
                        }
                        proto::postgresql::postgres_field::Field::Text(text) => text.text,
                        proto::postgresql::postgres_field::Field::Bytea(bytea) => {
                            bytea.bytea.map(|value| bytes_to_hex(&value))
                        }
                        proto::postgresql::postgres_field::Field::Date(date) => date.date,
                        proto::postgresql::postgres_field::Field::Time(time) => time.time,
                        proto::postgresql::postgres_field::Field::Timetz(timetz) => timetz.timetz,
                        proto::postgresql::postgres_field::Field::Timestamp(timestamp) => {
                            timestamp.datetime.map(|value| format!("{value:?}"))
                        }
                        proto::postgresql::postgres_field::Field::Timestamptz(timestamptz) => {
                            timestamptz.timestamptz.map(|value| format!("{value:?}"))
                        }
                        proto::postgresql::postgres_field::Field::Interval(interval) => {
                            interval.interval
                        }
                        proto::postgresql::postgres_field::Field::Boolean(boolean) => {
                            boolean.boolean.map(|v| v.to_string())
                        }
                        proto::postgresql::postgres_field::Field::Money(money) => money.money,
                        proto::postgresql::postgres_field::Field::Cidr(cidr) => cidr.cidr,
                        proto::postgresql::postgres_field::Field::Inet(inet) => inet.inet,
                        proto::postgresql::postgres_field::Field::Box(box_value) => box_value.r#box,
                        proto::postgresql::postgres_field::Field::Circle(circle) => circle.circle,
                        proto::postgresql::postgres_field::Field::Line(line) => line.line,
                        proto::postgresql::postgres_field::Field::Lseg(lseg) => lseg.lseg,
                        proto::postgresql::postgres_field::Field::Path(path) => path.path,
                        proto::postgresql::postgres_field::Field::Point(point) => point.point,
                        proto::postgresql::postgres_field::Field::Polygon(polygon) => {
                            polygon.polygon
                        }
                        proto::postgresql::postgres_field::Field::Json(json) => json.json,
                        proto::postgresql::postgres_field::Field::Jsonb(jsonb) => jsonb.jsonb,
                        proto::postgresql::postgres_field::Field::Tsquery(tsquery) => {
                            tsquery.tsquery
                        }
                        proto::postgresql::postgres_field::Field::Tsvector(tsvector) => {
                            tsvector.tsvector
                        }
                        proto::postgresql::postgres_field::Field::Uuid(uuid) => uuid.uuid,
                        proto::postgresql::postgres_field::Field::Xml(xml) => xml.xml,
                    },
                    _ => None,
                }
            }
        }

        impl proto::common::db_field::Field {
            pub fn field_from_string(
                input: &String,
                initial_field: &proto::common::db_field::Field,
            ) -> anyhow::Result<proto::common::db_field::Field> {
                match initial_field {
                    proto::common::db_field::Field::Postgres(
                        proto::postgresql::PostgresField { field: Some(field) },
                    ) => match field {
                        proto::postgresql::postgres_field::Field::Unsupported(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Unsupported(
                                proto::postgresql::Unsupported {
                                    value: if input.is_empty() {
                                        None
                                    } else {
                                        Some(input.clone())
                                    },
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Bigint(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Bigint(
                                proto::postgresql::Bigint {
                                    bigint: parse_optional(input)?,
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Bigserial(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Bigserial(
                                proto::postgresql::Bigserial {
                                    bigserial: parse_optional(input)?,
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::DoublePrecision(_) => {
                            Ok(postgres_field(
                                proto::postgresql::postgres_field::Field::DoublePrecision(
                                    proto::postgresql::DoublePrecision {
                                        double_precision: parse_optional(input)?,
                                    },
                                ),
                            ))
                        }
                        proto::postgresql::postgres_field::Field::Integer(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Integer(
                                proto::postgresql::Integer {
                                    integer: parse_optional(input)?,
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Numeric(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Numeric(
                                proto::postgresql::Numeric {
                                    numeric: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Real(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Real(
                                proto::postgresql::Real {
                                    real: parse_optional(input)?,
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Smallint(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Smallint(
                                proto::postgresql::Smallint {
                                    smallint: parse_optional(input)?,
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Smallserial(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Smallserial(
                                proto::postgresql::Smallserial {
                                    smallserial: parse_optional(input)?,
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Serial(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Serial(
                                proto::postgresql::Serial {
                                    serial: parse_optional(input)?,
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Bit(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Bit(proto::postgresql::Bit {
                                bit: optional_string(input),
                            }),
                        )),
                        proto::postgresql::postgres_field::Field::Varbit(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Varbit(
                                proto::postgresql::Varbit {
                                    varbit: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Char(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Char(
                                proto::postgresql::Char {
                                    r#char: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Varchar(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Varchar(
                                proto::postgresql::Varchar {
                                    varchar: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Text(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Text(
                                proto::postgresql::Text {
                                    text: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Bytea(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Bytea(
                                proto::postgresql::Bytea {
                                    bytea: parse_hex(input)?,
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Date(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Date(
                                proto::postgresql::Date {
                                    date: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Time(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Time(
                                proto::postgresql::Time {
                                    time: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Timetz(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Timetz(
                                proto::postgresql::Timetz {
                                    timetz: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Timestamp(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Timestamp(
                                proto::postgresql::Timestamp {
                                    datetime: parse_timestamp(input)?,
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Timestamptz(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Timestamptz(
                                proto::postgresql::Timestamptz {
                                    timestamptz: parse_timestamp(input)?,
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Interval(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Interval(
                                proto::postgresql::Interval {
                                    interval: optional_string(input),
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Boolean(_) => {
                            let boolean = match input.as_str() {
                                "" => None,
                                "true" | "t" | "1" => Some(true),
                                "false" | "f" | "0" => Some(false),
                                _ => anyhow::bail!("cannot be mapped into boolean"),
                            };
                            Ok(postgres_field(
                                proto::postgresql::postgres_field::Field::Boolean(
                                    proto::postgresql::Boolean { boolean },
                                ),
                            ))
                        }
                        proto::postgresql::postgres_field::Field::Money(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Money(
                                proto::postgresql::Money {
                                    money: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Cidr(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Cidr(
                                proto::postgresql::Cidr {
                                    cidr: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Inet(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Inet(
                                proto::postgresql::Inet {
                                    inet: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Box(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Box(proto::postgresql::Box {
                                r#box: optional_string(input),
                            }),
                        )),
                        proto::postgresql::postgres_field::Field::Circle(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Circle(
                                proto::postgresql::Circle {
                                    circle: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Line(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Line(
                                proto::postgresql::Line {
                                    line: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Lseg(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Lseg(
                                proto::postgresql::Lseg {
                                    lseg: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Path(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Path(
                                proto::postgresql::Path {
                                    path: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Point(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Point(
                                proto::postgresql::Point {
                                    point: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Polygon(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Polygon(
                                proto::postgresql::Polygon {
                                    polygon: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Json(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Json(
                                proto::postgresql::Json {
                                    json: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Jsonb(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Jsonb(
                                proto::postgresql::Jsonb {
                                    jsonb: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Tsquery(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Tsquery(
                                proto::postgresql::Tsquery {
                                    tsquery: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Tsvector(_) => Ok(
                            postgres_field(proto::postgresql::postgres_field::Field::Tsvector(
                                proto::postgresql::Tsvector {
                                    tsvector: optional_string(input),
                                },
                            )),
                        ),
                        proto::postgresql::postgres_field::Field::Uuid(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Uuid(
                                proto::postgresql::Uuid {
                                    uuid: optional_string(input),
                                },
                            ),
                        )),
                        proto::postgresql::postgres_field::Field::Xml(_) => Ok(postgres_field(
                            proto::postgresql::postgres_field::Field::Xml(proto::postgresql::Xml {
                                xml: optional_string(input),
                            }),
                        )),
                    },
                    proto::common::db_field::Field::Postgres(
                        proto::postgresql::PostgresField { field: None },
                    ) => Ok(proto::common::db_field::Field::Postgres(
                        proto::postgresql::PostgresField { field: None },
                    )),
                }
            }
        }
    }
}
