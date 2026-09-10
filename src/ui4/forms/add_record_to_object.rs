use std::sync::Arc;

use anyhow::anyhow;
use iocraft::{AnyElement, Color, Hooks, KeyCode, KeyEvent, KeyEventKind, Props, component, components::{Text, TextInput, View}, element, hooks::{State, UseState, UseTerminalEvents}};

use crate::core::proto;

#[derive(Default, Props)]
pub struct AddRecordToObjectFormProps {
    pub object: proto::common::DbObject,
}

#[component]
pub fn AddRecordToObjectForm(props: &AddRecordToObjectFormProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut focused_field: State::<Option<usize>> = hooks.use_state(|| None);
    let mut to_focus_field = hooks.use_state(|| 0usize);
    let fields = hooks.use_state(|| object_into_renderable(&props.object).unwrap_or_default());
    
    hooks.use_local_terminal_events(move |event| {
        match event {
            iocraft::TerminalEvent::Key(KeyEvent {code, kind, ..}) if kind != KeyEventKind::Release => match code {
                KeyCode::Esc if focused_field.read().is_some() => {
                    focused_field.set(None);
                }
                KeyCode::Char('j') | KeyCode::Down if focused_field.read().is_none() => {
                    let fields_count = fields.read().len();
                    to_focus_field.set(to_focus_field.get().saturating_add(1).min(fields_count.saturating_sub(1)));
                }
                KeyCode::Char('k') | KeyCode::Up if focused_field.read().is_none() => {
                    to_focus_field.set(to_focus_field.get().saturating_sub(1));
                }
                KeyCode::Char('i') if focused_field.read().is_none() => {
                    focused_field.set(Some(to_focus_field.get()));
                }
                KeyCode::Enter if focused_field.read().is_none() => {
                }
                _ => {}
            },
            _ => {},
        }
    });

    element! {
        View(width: 100pct, flex_direction: iocraft::FlexDirection::Column) {
            #(
                fields.read().iter().enumerate().map(|(i, f)| {
                    element! {
                        View (border_style: if to_focus_field.get() == i { iocraft::components::BorderStyle::Classic } else { iocraft::components::BorderStyle::None }) {
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
    fields: Option<State<Vec<(String, Option<proto::common::DbField>)>>>,
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
        .get(index)
        .and_then(|(_, field)| field.as_ref())
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
                        let mut next_fields = fields.read().clone();

                        if let Some((_, Some(field))) = next_fields.get_mut(index) {
                            if let Some(initial_field) = field.field.clone() {
                                if let Ok(mapped_field) = field_from_string(&new_value, initial_field) {
                                    field.field = Some(mapped_field);
                                    fields.set(next_fields);
                                }
                            }
                        }
                    }
                )
            }
        }
    }
}

fn field_into_string(input: &proto::common::db_field::Field) -> String {
    match input {
        proto::common::db_field::Field::Str(s) => s.clone(),
        proto::common::db_field::Field::StrContainer(sc) => sc.strs.join("\n"),
        proto::common::db_field::Field::I8(i) => i.to_string(),
        proto::common::db_field::Field::I16(i) => i.to_string(),
        proto::common::db_field::Field::I32(i) => i.to_string(),
        proto::common::db_field::Field::I64(i) => i.to_string(),
        proto::common::db_field::Field::Boolean(b) => b.to_string(),
        proto::common::db_field::Field::Datetime(timestamp) => timestamp.to_string(),
    }
}

fn field_from_string(input: &String, initial_field: proto::common::db_field::Field) -> anyhow::Result<proto::common::db_field::Field> {
    match initial_field {
        proto::common::db_field::Field::Str(_) => {
            Ok(proto::common::db_field::Field::Str(input.clone()))
        },
        proto::common::db_field::Field::StrContainer(_) => {
            let splitted: Vec<String> = input.split('\n').map(|s| s.to_string()).collect();
            Ok(proto::common::db_field::Field::StrContainer(proto::common::StringContainer {
                strs: splitted
            }))
        },
        proto::common::db_field::Field::I8(_) => {
            if let Ok(i) = i8::from_str_radix(&input, 10) {
                Ok(proto::common::db_field::Field::I8(i as i32))
            } else if let Ok(i) = i8::from_str_radix(&input, 16) {
                Ok(proto::common::db_field::Field::I8(i as i32))
            } else {
                Ok(proto::common::db_field::Field::I8(i8::from_str_radix(&input, 2)? as i32))
            }
        },
        proto::common::db_field::Field::I16(_) => {
            if let Ok(i) = i16::from_str_radix(&input, 10) {
                Ok(proto::common::db_field::Field::I16(i as i32))
            } else if let Ok(i) = i16::from_str_radix(&input, 16) {
                Ok(proto::common::db_field::Field::I16(i as i32))
            } else {
                Ok(proto::common::db_field::Field::I16(i16::from_str_radix(&input, 2)? as i32))
            }
        },
        proto::common::db_field::Field::I32(_) => {
            if let Ok(i) = i32::from_str_radix(&input, 10) {
                Ok(proto::common::db_field::Field::I32(i))
            } else if let Ok(i) = i32::from_str_radix(&input, 16) {
                Ok(proto::common::db_field::Field::I32(i))
            } else {
                Ok(proto::common::db_field::Field::I32(i32::from_str_radix(&input, 2)?))
            }
        },
        proto::common::db_field::Field::I64(_) => {
            if let Ok(i) = i64::from_str_radix(&input, 10) {
                Ok(proto::common::db_field::Field::I64(i))
            } else if let Ok(i) = i64::from_str_radix(&input, 16) {
                Ok(proto::common::db_field::Field::I64(i))
            } else {
                Ok(proto::common::db_field::Field::I64(i64::from_str_radix(&input, 2)?))
            }
        },
        proto::common::db_field::Field::Boolean(_) => {
            match input.as_str() {
                "true" | "1" => Ok(proto::common::db_field::Field::Boolean(true)),
                "false" | "0" => Ok(proto::common::db_field::Field::Boolean(false)),
                _ => Err(anyhow!("cannot be mapped into boolean"))
            }
        },
        proto::common::db_field::Field::Datetime(_) => {
            todo!()
        },
    }
}

fn object_into_renderable(obj: &proto::common::DbObject) -> Option<Vec<(String, Option<proto::common::DbField>)>> {
    let specification = obj.specification.as_ref()?;

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
