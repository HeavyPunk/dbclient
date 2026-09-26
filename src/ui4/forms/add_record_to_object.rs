use anyhow::anyhow;
use iocraft::{
    component,
    components::{Text, TextInput, View},
    element,
    hooks::{State, UseState, UseTerminalEvents},
    AnyElement, Color, Hooks, KeyCode, KeyEvent, KeyEventKind, Props,
};

use crate::{core::proto, ui4::app_state::QueryResultCmd};

#[derive(Default, Props)]
pub struct AddRecordToObjectFormProps {
    pub object: proto::common::DbObject,
    pub cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
}

#[component]
pub fn AddRecordToObjectForm(
    props: &AddRecordToObjectFormProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut focused_field: State<Option<usize>> = hooks.use_state(|| None);
    let mut to_focus_field = hooks.use_state(|| 0usize);
    // Keep the complete specification in state so downstream components retain
    // the object's context (descriptor, constraints, indexes, etc.).
    let fields = hooks.use_state(|| props.object.specification.clone());
    let renderable_fields =
        specification_into_renderable(fields.read().as_ref()).unwrap_or_default();
    let fields_count = renderable_fields.len();
    let cmd_pipe = props.cmd_pipe.clone();

    hooks.use_local_terminal_events(move |event| match event {
        iocraft::TerminalEvent::Key(KeyEvent { code, kind, .. })
            if kind != KeyEventKind::Release =>
        {
            match code {
                KeyCode::Esc if focused_field.read().is_none() => {
                    tokio::task::block_in_place(|| {
                        cmd_pipe
                            .as_ref()
                            .and_then(|ch| Some(ch.blocking_send(QueryResultCmd::ClosePopup)))
                    });
                }
                KeyCode::Esc if focused_field.read().is_some() => {
                    focused_field.set(None);
                }
                KeyCode::Char('j') | KeyCode::Down if focused_field.read().is_none() => {
                    to_focus_field.set(
                        to_focus_field
                            .get()
                            .saturating_add(1)
                            .min(fields_count.saturating_sub(1)),
                    );
                }
                KeyCode::Char('k') | KeyCode::Up if focused_field.read().is_none() => {
                    to_focus_field.set(to_focus_field.get().saturating_sub(1));
                }
                KeyCode::Char('i') if focused_field.read().is_none() => {
                    focused_field.set(Some(to_focus_field.get()));
                }
                KeyCode::Enter if focused_field.read().is_none() => {
                    let record = fields.read().as_ref().and_then(specification_into_record);

                    if let Some(record) = record {
                        tokio::task::block_in_place(|| {
                            cmd_pipe.as_ref().and_then(|ch| {
                                Some(ch.blocking_send(QueryResultCmd::AddRecordToObject(record)))
                            })
                        });
                    }
                }
                _ => {}
            }
        }
        _ => {}
    });

    element! {
        View(
            width: 100pct,
            flex_direction: iocraft::FlexDirection::Column,
            border_style: iocraft::components::BorderStyle::Round,
            border_color: Color::Yellow,
        ) {
            #(
                renderable_fields.iter().enumerate().map(|(i, f)| {
                    element! {
                        View (border_style: if to_focus_field.get() == i { iocraft::components::BorderStyle::Single } else { iocraft::components::BorderStyle::None }) {
                            FormTextInput(
                                has_focus: if let Some(ff) = focused_field.read().as_ref() { *ff == i } else { false },
                                field_name: f.0.clone(),
                                fields: Some(fields),
                                index: i,
                            )
                        }
                    }
                })
            )
        }
    }
}

#[derive(Default, Props)]
struct FormTextInputProps {
    has_focus: bool,
    field_name: String,
    fields: Option<State<Option<proto::common::db_object::Specification>>>,
    index: usize,
}

#[component]
fn FormTextInput(props: &FormTextInputProps, _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let Some(mut fields) = props.fields else {
        panic!("fields is required");
    };
    let index = props.index;
    let value = fields
        .read()
        .as_ref()
        .and_then(|specification| field_at(specification, index))
        .and_then(|field| field.field.as_ref())
        .map(field_into_string)
        .unwrap_or_default();

    element! {
        View (width: 100pct) {
            View(margin_right: 1) {
                Text(content: props.field_name.clone())
            }
            View(
                background_color: if props.has_focus { Some(Color::DarkGrey) } else { None },
                width: 90pct,
                height: 1,
            ) {
                TextInput(
                    has_focus: props.has_focus,
                    value,
                    on_change: move |new_value| {
                        let mut next_specification = fields.read().clone();

                        if let Some(field) = next_specification
                            .as_mut()
                            .and_then(|specification| field_at_mut(specification, index))
                        {
                            if let Some(initial_field) = &field.field {
                                if let Ok(mapped_field) = field_from_string(&new_value, initial_field) {
                                    field.field = Some(mapped_field);
                                    fields.set(next_specification);
                                }
                            }
                        }
                    }
                )
            }
        }
    }
}

pub fn field_into_string(input: &proto::common::db_field::Field) -> String {
    match input {
        proto::common::db_field::Field::Str(s) => {
            s.str.as_ref().map_or(String::default(), |v| v.to_string())
        }
        proto::common::db_field::Field::StrContainer(sc) => sc.strs.join("\n"),
        proto::common::db_field::Field::I8(i) => i.i8.map_or(String::default(), |v| v.to_string()),
        proto::common::db_field::Field::I16(i) => {
            i.i16.map_or(String::default(), |v| v.to_string())
        }
        proto::common::db_field::Field::I32(i) => {
            i.i32.map_or(String::default(), |v| v.to_string())
        }
        proto::common::db_field::Field::I64(i) => {
            i.i64.map_or(String::default(), |v| v.to_string())
        }
        proto::common::db_field::Field::Boolean(b) => {
            b.boolean.map_or(String::default(), |v| v.to_string())
        }
        proto::common::db_field::Field::Datetime(timestamp) => timestamp
            .datetime
            .map_or(String::default(), |v| v.to_string()),
    }
}

pub fn field_from_string(
    input: &String,
    initial_field: &proto::common::db_field::Field,
) -> anyhow::Result<proto::common::db_field::Field> {
    match initial_field {
        proto::common::db_field::Field::Str(_) => {
            Ok(proto::common::db_field::Field::Str(proto::common::String {
                str: if input.is_empty() {
                    None
                } else {
                    Some(input.clone())
                },
            }))
        }
        proto::common::db_field::Field::StrContainer(_) => {
            let splitted: Vec<String> = input.split('\n').map(|s| s.to_string()).collect();
            Ok(proto::common::db_field::Field::StrContainer(
                proto::common::StringContainer { strs: splitted },
            ))
        }
        proto::common::db_field::Field::I8(_) => {
            if input.is_empty() {
                return Ok(proto::common::db_field::Field::I8(proto::common::Int8 {
                    i8: None,
                }));
            }
            let i = i8::from_str_radix(&input, 10)?;
            Ok(proto::common::db_field::Field::I8(proto::common::Int8 {
                i8: Some(i as i32),
            }))
        }
        proto::common::db_field::Field::I16(_) => {
            if input.is_empty() {
                return Ok(proto::common::db_field::Field::I16(proto::common::Int16 {
                    i16: None,
                }));
            }
            let i = i16::from_str_radix(&input, 10)?;
            Ok(proto::common::db_field::Field::I16(proto::common::Int16 {
                i16: Some(i as i32),
            }))
        }
        proto::common::db_field::Field::I32(_) => {
            if input.is_empty() {
                return Ok(proto::common::db_field::Field::I32(proto::common::Int32 {
                    i32: None,
                }));
            }
            let i = i32::from_str_radix(&input, 10)?;
            Ok(proto::common::db_field::Field::I32(proto::common::Int32 {
                i32: Some(i),
            }))
        }
        proto::common::db_field::Field::I64(_) => {
            if input.is_empty() {
                return Ok(proto::common::db_field::Field::I64(proto::common::Int64 {
                    i64: None,
                }));
            }
            let i = i64::from_str_radix(&input, 10)?;
            Ok(proto::common::db_field::Field::I64(proto::common::Int64 {
                i64: Some(i),
            }))
        }
        proto::common::db_field::Field::Boolean(_) => match input.as_str() {
            "true" | "1" => Ok(proto::common::db_field::Field::Boolean(
                proto::common::Boolean {
                    boolean: Some(true),
                },
            )),
            "false" | "0" => Ok(proto::common::db_field::Field::Boolean(
                proto::common::Boolean {
                    boolean: Some(false),
                },
            )),
            "" => Ok(proto::common::db_field::Field::Boolean(
                proto::common::Boolean { boolean: None },
            )),
            _ => Err(anyhow!("cannot be mapped into boolean")),
        },
        proto::common::db_field::Field::Datetime(_) => {
            todo!()
        }
    }
}

fn specification_into_record(
    specification: &proto::common::db_object::Specification,
) -> Option<proto::common::DbRecord> {
    let proto::common::db_object::Specification::Postgres(postgres) = specification else {
        return None;
    };

    let proto::common::postgres_object::Object::Table(table) = postgres.object.as_ref()? else {
        return None;
    };

    Some(proto::common::DbRecord {
        specification: Some(proto::common::db_record::Specification::Postgres(
            proto::common::PostgresRecord {
                record: Some(proto::common::postgres_record::Record::Table(
                    proto::common::PostgresRecordTable {
                        rows: vec![proto::common::PostgresRecordTableRow {
                            columns: table.columns.clone(),
                            values: table
                                .columns
                                .iter()
                                .map(|column| column.field.clone().unwrap_or_default())
                                .collect(),
                        }],
                    },
                )),
            },
        )),
    })
}

fn specification_into_renderable(
    specification: Option<&proto::common::db_object::Specification>,
) -> Option<Vec<(String, Option<proto::common::DbField>)>> {
    let specification = specification?;
    match specification {
        proto::common::db_object::Specification::Postgres(postgres) => {
            match postgres.object.as_ref()? {
                proto::common::postgres_object::Object::Table(table) => Some(
                    table
                        .columns
                        .iter()
                        .map(|column| (column.name.clone(), column.field.clone()))
                        .collect(),
                ),
                // A record can be added only to a table, not to a database or schema.
                proto::common::postgres_object::Object::Database(_)
                | proto::common::postgres_object::Object::Schema(_) => None,
            }
        }
        // Redis objects are not implemented yet.
        proto::common::db_object::Specification::Redis(_) => None,
    }
}

fn field_at(
    specification: &proto::common::db_object::Specification,
    index: usize,
) -> Option<&proto::common::DbField> {
    match specification {
        proto::common::db_object::Specification::Postgres(postgres) => {
            match postgres.object.as_ref()? {
                proto::common::postgres_object::Object::Table(table) => table
                    .columns
                    .get(index)
                    .and_then(|column| column.field.as_ref()),
                proto::common::postgres_object::Object::Database(_)
                | proto::common::postgres_object::Object::Schema(_) => None,
            }
        }
        proto::common::db_object::Specification::Redis(_) => None,
    }
}

fn field_at_mut(
    specification: &mut proto::common::db_object::Specification,
    index: usize,
) -> Option<&mut proto::common::DbField> {
    match specification {
        proto::common::db_object::Specification::Postgres(postgres) => {
            match postgres.object.as_mut()? {
                proto::common::postgres_object::Object::Table(table) => table
                    .columns
                    .get_mut(index)
                    .and_then(|column| column.field.as_mut()),
                proto::common::postgres_object::Object::Database(_)
                | proto::common::postgres_object::Object::Schema(_) => None,
            }
        }
        proto::common::db_object::Specification::Redis(_) => None,
    }
}
