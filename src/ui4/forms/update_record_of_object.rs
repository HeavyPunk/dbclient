use iocraft::{
    component,
    components::{ScrollView, ScrollViewHandle, Text, TextInput, View},
    element,
    hooks::{State, UseComponentRect, UseRef, UseState, UseTerminalEvents},
    AnyElement, Color, Hooks, JustifyContent, KeyCode, KeyEvent, KeyEventKind, Props,
};

use crate::{core::proto, ui4::app_state::QueryResultCmd};

#[derive(Default, Props)]
pub struct UpdateRecordOfObjectFormProps {
    pub initial_record: proto::common::DbRecord,
    pub cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
}

#[derive(Clone, Default)]
struct RecordField {
    name: String,
    value: proto::common::DbField,
    draft: String,
}

#[component]
pub fn UpdateRecordOfObject(
    props: &UpdateRecordOfObjectFormProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let old_record = props.initial_record.clone();
    let fields = hooks.use_state(|| record_fields(&props.initial_record));
    let mut focused_field: State<Option<usize>> = hooks.use_state(|| None);
    let mut to_focus_field = hooks.use_state(|| 0usize);
    let fields_count = fields.read().len();
    let mut scroll_handle = hooks.use_ref_default::<ScrollViewHandle>();
    let field_heights = hooks.use_state(|| vec![1usize; fields_count]);
    let cmd_pipe = props.cmd_pipe.clone();
    let fields_for_submit = fields.clone();

    let selected_index = to_focus_field.get();
    let field_heights_ref = field_heights.read();
    let selected_top = (0..selected_index)
        .map(|index| field_heights_ref.get(index).copied().unwrap_or(1) + 2) // NOTE: +2 is borders
        .sum::<usize>();
    let selected_height = field_heights_ref.get(selected_index).copied().unwrap_or(1) + 2; // NOTE:
                                                                                           // +2 is
                                                                                           // borders
    let viewport_height = scroll_handle.read().viewport_height().max(0) as usize;
    let current_offset = scroll_handle.read().scroll_offset().max(0) as usize;
    if viewport_height > 0 {
        let selected_bottom = selected_top + selected_height;
        let next_offset = if selected_top < current_offset {
            selected_top
        } else if selected_bottom > current_offset + viewport_height {
            selected_bottom.saturating_sub(viewport_height)
        } else {
            current_offset
        };
        if next_offset != current_offset {
            scroll_handle.write().scroll_to(next_offset as i32);
        }
    }

    hooks.use_local_terminal_events(move |event| match event {
        iocraft::TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind == KeyEventKind::Press => {
            match code {
                KeyCode::Esc if focused_field.read().is_some() => focused_field.set(None),
                KeyCode::Esc => {
                    tokio::task::block_in_place(|| {
                        cmd_pipe
                            .as_ref()
                            .and_then(|ch| Some(ch.blocking_send(QueryResultCmd::ClosePopup)))
                    });
                }
                KeyCode::Char('j') | KeyCode::Down if focused_field.read().is_none() => {
                    if fields_count > 0 {
                        to_focus_field.set((to_focus_field.get() + 1).min(fields_count - 1));
                    }
                }
                KeyCode::Char('k') | KeyCode::Up if focused_field.read().is_none() => {
                    to_focus_field.set(to_focus_field.get().saturating_sub(1));
                }
                KeyCode::Char('i') if focused_field.read().is_none() && fields_count > 0 => {
                    focused_field.set(Some(to_focus_field.get()));
                }
                KeyCode::Enter if focused_field.read().is_none() => {
                    let mut committed_fields = fields_for_submit.read().clone();
                    let valid = committed_fields.iter_mut().all(|field| {
                        let Some(initial_field) = field.value.field.as_ref() else {
                            return false;
                        };
                        let Ok(new_field) =
                            crate::ui4::forms::add_record_to_object::field_from_string(
                                &field.draft,
                                initial_field,
                            )
                        else {
                            log::warn!(target: "update_record_of_object", "Invalid field value: `{}`", &field.draft);
                            return false;
                        };
                        field.value.field = Some(new_field);
                        true
                    });
                    if !valid {
                        return;
                    }
                    let new_record = fields_into_record(&old_record, &committed_fields);
                    tokio::task::block_in_place(|| {
                        cmd_pipe.as_ref().and_then(|ch| {
                            Some(ch.blocking_send(QueryResultCmd::UpdateRecordOfObject(
                                old_record.clone(),
                                new_record,
                            )))
                        })
                    });
                }
                _ => {}
            }
        }
        _ => {}
    });

    element! {
        View(
            width: 100pct,
            height: 100pct,
            flex_direction: iocraft::FlexDirection::Column,
            border_style: iocraft::components::BorderStyle::Round,
            border_color: Color::Yellow
        ) {
            View (
                justify_content: JustifyContent::Center
            ){
                Text(content: "Update record of object")
            }
            ScrollView(
                handle: Some(scroll_handle),
                scrollbar: Some(true),
                keyboard_scroll: Some(false),
            ) {
                View(
                    flex_direction: iocraft::FlexDirection::Column,
                    width: 100pct,
                ) {
                    #(fields.read().iter().enumerate().map(|(index, field)| element! {
                        View(
                            width: 100pct,
                            height: 100pct,
                            border_style: if to_focus_field.get() == index {
                                iocraft::components::BorderStyle::Single
                            } else {
                                iocraft::components::BorderStyle::None
                            },
                            padding: if to_focus_field.get() == index {
                                iocraft::Padding::Unset
                            } else {
                                iocraft::Padding::Length(1)
                            },
                        ) {
                            FormTextInput(
                                has_focus: focused_field.read().as_ref() == Some(&index),
                                field: field.clone(),
                                fields: Some(fields),
                                heights: Some(field_heights),
                                index,
                            )
                        }
                    }))
                }
            }
        }
    }
}

#[derive(Default, Props)]
struct FormTextInputProps {
    has_focus: bool,
    field: RecordField,
    fields: Option<State<Vec<RecordField>>>,
    heights: Option<State<Vec<usize>>>,
    index: usize,
}

#[component]
fn FormTextInput(props: &FormTextInputProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let Some(mut fields) = props.fields else {
        return element! { View() };
    };
    let Some(mut heights) = props.heights else {
        return element! { View() };
    };
    let index = props.index;
    let value = fields
        .read()
        .get(index)
        .map(|field| field.draft.clone())
        .unwrap_or_default();

    // NOTE: layout_ready is used to prevent invalid TextInput render on the first time render.
    // Multiline mode should be enabled only after fully built widget layout
    let mut layout_ready = false;
    if let Some(rect) = hooks.use_component_rect() {
        layout_ready = true;
        let height = rect.bottom.saturating_sub(rect.top) as usize;
        let mut next_heights = heights.read().clone();
        if next_heights.get(index).copied() != Some(height) {
            if let Some(current_height) = next_heights.get_mut(index) {
                *current_height = height;
                heights.set(next_heights);
            }
        }
    }

    element! {
        View(width: 100pct) {
            View(
                flex_direction: iocraft::FlexDirection::Column,
                margin_right: 1
            ) {
                View {
                    Text(
                        content: props.field.name.clone(),
                        color: Color::Blue
                    )
                }
                View {
                    Text(
                        content: if let Some(s) = field_type_into_string(&props.field.value) {
                            format!("({s})")
                        } else {
                            String::default()
                        },
                        color: Color::DarkGrey
                    )
                }
            }
            View(
                background_color: if props.has_focus { Some(Color::DarkGrey) } else { None },
                width: 80pct,
            ) {
                TextInput(
                    has_focus: props.has_focus,
                    value,
                    multiline: layout_ready,
                    auto_grow: layout_ready,
                    on_change: move |new_value| {
                        let mut next_fields = fields.read().clone();
                        let Some(field) = next_fields.get_mut(index) else { return; };
                        field.draft = new_value;
                        fields.set(next_fields);
                    }
                )
            }
        }
    }
}

/// Converts a record into the neutral representation consumed by the form.
///
/// The UI only deals with `RecordField`; the protobuf-specific representation
/// is kept in this adapter and can be extended when another `DbRecord` variant
/// is added.
fn record_fields(record: &proto::common::DbRecord) -> Vec<RecordField> {
    let Some(proto::common::db_record::Specification::Postgres(postgres)) =
        record.specification.as_ref()
    else {
        return vec![];
    };
    let Some(proto::postgresql::postgres_record::Record::Table(table)) = postgres.record.as_ref()
    else {
        return vec![];
    };
    let Some(row) = table.rows.first() else {
        return vec![];
    };
    table
        .columns
        .iter()
        .zip(row.values.iter())
        .map(|(column, value)| RecordField {
            name: column.name.clone(),
            value: proto::common::DbField {
                field: Some(proto::common::db_field::Field::Postgres(value.clone())),
            },
            draft: value
                .field
                .as_ref()
                .map(|field| {
                    crate::ui4::forms::add_record_to_object::field_into_string(
                        &proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
                            field: Some(field.clone()),
                        }),
                    )
                })
                .unwrap_or_default(),
        })
        .collect()
}

fn fields_into_record(
    record: &proto::common::DbRecord,
    fields: &[RecordField],
) -> proto::common::DbRecord {
    let mut result = record.clone();
    if let Some(proto::common::db_record::Specification::Postgres(postgres)) =
        result.specification.as_mut()
    {
        if let Some(proto::postgresql::postgres_record::Record::Table(table)) =
            postgres.record.as_mut()
        {
            if let Some(row) = table.rows.first_mut() {
                row.values = fields
                    .iter()
                    .map(|field| match field.value.field.as_ref() {
                        Some(proto::common::db_field::Field::Postgres(value)) => value.clone(),
                        None => proto::postgresql::DbField::default(),
                    })
                    .collect();
            }
        }
    }
    result
}

pub fn field_type_into_string(field: &proto::common::DbField) -> Option<&'static str> {
    match field.field {
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::I8(_)),
        })) => Some("int8"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::I16(_)),
        })) => Some("int16"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::I32(_)),
        })) => Some("int32"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::I64(_)),
        })) => Some("int64"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::Str(_)),
        })) => Some("string"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::StrContainer(_)),
        })) => Some("text"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::Boolean(_)),
        })) => Some("bool"),
        Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: Some(proto::postgresql::db_field::Field::Datetime(_)),
        })) => Some("datetime"),
        None
        | Some(proto::common::db_field::Field::Postgres(proto::postgresql::DbField {
            field: None,
        })) => None,
    }
}
