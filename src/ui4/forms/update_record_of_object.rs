use iocraft::{
    component,
    components::{Text, TextInput, View},
    element,
    hooks::{State, UseState, UseTerminalEvents},
    AnyElement, Color, Hooks, KeyCode, KeyEvent, KeyEventKind, Props,
};

use crate::{
    core::proto,
    ui4::app_state::{AppState, QueryResultCmd},
};

#[derive(Default, Props)]
pub struct UpdateRecordOfObjectFormProps {
    pub initial_record: proto::common::DbRecord,
    pub cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
}

#[derive(Clone, Default)]
struct RecordField {
    name: String,
    value: proto::common::DbField,
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
    let cmd_pipe = props.cmd_pipe.clone();

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
                    let new_record = fields_into_record(&old_record, &fields.read());
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
            flex_direction: iocraft::FlexDirection::Column,
            border_style: iocraft::components::BorderStyle::Round,
            border_color: Color::Yellow
        ) {
            #(fields.read().iter().enumerate().map(|(index, field)| element! {
                View(
                    border_style: if to_focus_field.get() == index {
                        iocraft::components::BorderStyle::Single
                    } else { iocraft::components::BorderStyle::None }
                ) {
                    FormTextInput(
                        has_focus: focused_field.read().as_ref() == Some(&index),
                        field: field.clone(), fields: Some(fields), index,
                    )
                }
            }))
        }
    }
}

#[derive(Default, Props)]
struct FormTextInputProps {
    has_focus: bool,
    field: RecordField,
    fields: Option<State<Vec<RecordField>>>,
    index: usize,
}

#[component]
fn FormTextInput(props: &FormTextInputProps, _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let Some(mut fields) = props.fields else {
        return element! { View() };
    };
    let index = props.index;
    let value = fields
        .read()
        .get(index)
        .and_then(|field| field.value.field.as_ref())
        .map(crate::ui4::forms::add_record_to_object::field_into_string)
        .unwrap_or_default();

    element! {
        View(width: 100pct) {
            View(margin_right: 1) {
                Text(
                    content: props.field.name.clone(),
                    color: Color::Blue
                )
            }
            View(
                background_color: if props.has_focus { Some(Color::DarkGrey) } else { None },
                width: 90pct, height: 1,
            ) {
                TextInput(
                    has_focus: props.has_focus, value,
                    on_change: move |new_value| {
                        let mut next_fields = fields.read().clone();
                        let Some(field) = next_fields.get_mut(index) else { return; };
                        let Some(initial_field) = field.value.field.as_ref() else { return; };
                        let Ok(new_field) = crate::ui4::forms::add_record_to_object::field_from_string(
                            &new_value, initial_field,
                        ) else { return; };
                        field.value.field = Some(new_field);
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
    let Some(proto::common::postgres_record::Record::Table(table)) = postgres.record.as_ref()
    else {
        return vec![];
    };
    let Some(row) = table.rows.first() else {
        return vec![];
    };
    row.columns
        .iter()
        .zip(row.values.iter())
        .map(|(column, value)| RecordField {
            name: column.name.clone(),
            value: value.clone(),
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
        if let Some(proto::common::postgres_record::Record::Table(table)) = postgres.record.as_mut()
        {
            if let Some(row) = table.rows.first_mut() {
                row.values = fields.iter().map(|field| field.value.clone()).collect();
            }
        }
    }
    result
}
