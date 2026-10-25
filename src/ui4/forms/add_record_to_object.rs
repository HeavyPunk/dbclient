use iocraft::{
    component,
    components::{ScrollView, ScrollViewHandle, Text, TextInput, View},
    element,
    hooks::{State, UseComponentRect, UseRef, UseState, UseTerminalEvents},
    AnyElement, Color, Hooks, JustifyContent, KeyCode, KeyEvent, KeyEventKind, Props,
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
    let mut scroll_handle = hooks.use_ref_default::<ScrollViewHandle>();
    let field_heights = hooks.use_state(|| vec![1usize; fields_count]);
    let drafts = hooks.use_state(|| {
        renderable_fields
            .iter()
            .map(|(_, field)| {
                field
                    .as_ref()
                    .and_then(|f| f.field.as_ref())
                    .map(|f| -> Option<String> { proto::common::db_field::Field::into(f.clone()) })
                    .unwrap_or_default()
                    .unwrap_or_default()
            })
            .collect::<Vec<String>>()
    });
    let cmd_pipe = props.cmd_pipe.clone();
    let fields_for_submit = fields.clone();
    let drafts_for_submit = drafts.clone();

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
                    let mut specification = fields_for_submit.read().clone();
                    let Some(mut specification) = specification.as_mut() else {
                        return;
                    };
                    let drafts = drafts_for_submit.read();
                    let valid = drafts.iter().enumerate().all(|(index, input)| {
                        let Some(initial_field) = field_at(&specification, index) else {
                            return false;
                        };
                        let Some(initial_field) = initial_field.field.as_ref() else {
                            return false;
                        };
                        let Ok(value) = proto::common::db_field::Field::field_from_string(input, initial_field) else {
                            log::warn!(target: "add_record_to_object", "Invalid field value: `{}`", input);
                            return false;
                        };
                        let Some(field) = field_at_mut_postgres(&mut specification, index) else {
                            return false;
                        };
                        let proto::common::db_field::Field::Postgres(value) = value;
                        field.field = value.field;
                        true
                    });
                    if !valid {
                        return;
                    }
                    let record = specification_into_record(specification);

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
            height: 100pct,
            flex_direction: iocraft::FlexDirection::Column,
            border_style: iocraft::components::BorderStyle::Round,
            border_color: Color::Yellow,
        ) {
            View (
                justify_content: JustifyContent::Center
            ){
                Text(content: "Add record to object")
            }
            ScrollView(
                handle: Some(scroll_handle),
                scrollbar: Some(true),
                keyboard_scroll: Some(false),
            ) {
                View(flex_direction: iocraft::FlexDirection::Column, width: 100pct) {
                    #(
                        renderable_fields.iter().enumerate().map(|(i, f)| {
                            element! {
                                View(
                                    width: 100pct,
                                    height: 100pct,
                                    border_style: if to_focus_field.get() == i {
                                        iocraft::components::BorderStyle::Single
                                    } else {
                                        iocraft::components::BorderStyle::None
                                    },
                                    padding: if to_focus_field.get() == i {
                                        iocraft::Padding::Unset
                                    } else {
                                        iocraft::Padding::Length(1)
                                    },
                                ) {
                                    FormTextInput(
                                        has_focus: if let Some(ff) = focused_field.read().as_ref() { *ff == i } else { false },
                                        field_name: f.0.clone(),
                                        field_meta: f.1.clone(),
                                        drafts: Some(drafts),
                                        heights: Some(field_heights),
                                        index: i,
                                    )
                                }
                            }
                        })
                    )
                }
            }
        }
    }
}

#[derive(Default, Props)]
struct FormTextInputProps {
    has_focus: bool,
    field_name: String,
    field_meta: Option<proto::common::DbField>,
    drafts: Option<State<Vec<String>>>,
    heights: Option<State<Vec<usize>>>,
    index: usize,
}

#[component]
fn FormTextInput(props: &FormTextInputProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let Some(mut drafts) = props.drafts else {
        panic!("drafts is required");
    };
    let Some(mut heights) = props.heights else {
        panic!("heights is required");
    };
    let index = props.index;
    let value = drafts.read().get(index).cloned().unwrap_or_default();

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
        View (width: 100pct) {
            View(
                flex_direction: iocraft::FlexDirection::Column,
                margin_right: 1,
            ) {
                View {
                    Text(
                        content: props.field_name.clone(),
                        color: Color::Blue
                    )
                }
                View {
                    Text(
                        content: if let Some(s) = props.field_meta.as_ref().and_then(|fm| super::update_record_of_object::field_type_into_string(fm)) {
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
                        let mut next_drafts = drafts.read().clone();
                        if let Some(draft) = next_drafts.get_mut(index) {
                            *draft = new_value;
                            drafts.set(next_drafts);
                        }
                    }
                )
            }
        }
    }
}

fn specification_into_record(
    specification: &proto::common::db_object::Specification,
) -> Option<proto::common::DbRecord> {
    let proto::common::db_object::Specification::Postgres(postgres) = specification else {
        return None;
    };

    let proto::postgresql::postgres_object::Object::Table(table) = postgres.object.as_ref()? else {
        return None;
    };

    Some(proto::common::DbRecord {
        specification: Some(proto::common::db_record::Specification::Postgres(
            proto::postgresql::PostgresRecord {
                record: Some(proto::postgresql::postgres_record::Record::Table(
                    proto::postgresql::PostgresRecordTable {
                        columns: table.columns.clone(),
                        rows: vec![proto::postgresql::PostgresRecordTableRow {
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
                proto::postgresql::postgres_object::Object::Table(table) => Some(
                    table
                        .columns
                        .iter()
                        .map(|column| {
                            (
                                column.name.clone(),
                                column.field.clone().map(|f| proto::common::DbField {
                                    field: Some(proto::common::db_field::Field::Postgres(f)),
                                }),
                            )
                        })
                        .collect(),
                ),
                // A record can be added only to a table, not to a database or schema.
                proto::postgresql::postgres_object::Object::Database(_)
                | proto::postgresql::postgres_object::Object::Schema(_) => None,
            }
        }
        // Redis objects are not implemented yet.
        proto::common::db_object::Specification::Redis(_) => None,
    }
}

fn field_at(
    specification: &proto::common::db_object::Specification,
    index: usize,
) -> Option<proto::common::DbField> {
    match specification {
        proto::common::db_object::Specification::Postgres(postgres) => {
            match postgres.object.as_ref()? {
                proto::postgresql::postgres_object::Object::Table(table) => {
                    table.columns.get(index).and_then(|column| {
                        column.field.clone().map(|f| proto::common::DbField {
                            field: Some(proto::common::db_field::Field::Postgres(f)),
                        })
                    })
                }
                proto::postgresql::postgres_object::Object::Database(_)
                | proto::postgresql::postgres_object::Object::Schema(_) => None,
            }
        }
        proto::common::db_object::Specification::Redis(_) => None,
    }
}

fn field_at_mut_postgres(
    specification: &mut proto::common::db_object::Specification,
    index: usize,
) -> Option<&mut proto::postgresql::PostgresField> {
    match specification {
        proto::common::db_object::Specification::Postgres(postgres) => {
            match postgres.object.as_mut()? {
                proto::postgresql::postgres_object::Object::Table(table) => table
                    .columns
                    .get_mut(index)
                    .and_then(|column| column.field.as_mut()),
                proto::postgresql::postgres_object::Object::Database(_)
                | proto::postgresql::postgres_object::Object::Schema(_) => None,
            }
        }
        proto::common::db_object::Specification::Redis(_) => None,
    }
}
