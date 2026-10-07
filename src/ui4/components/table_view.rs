use std::sync::{Arc, Mutex};

use iocraft::{
    component,
    components::{BorderStyle, ScrollView, ScrollViewHandle, Text, View},
    element,
    hooks::{UseRef, UseState},
    AnyElement, Color, Edges, FlexDirection, Hooks, KeyCode, KeyEventKind, KeyModifiers, Props,
};

use crate::{core::proto::common::db_field::Field, ui4::control::UseHotkeys};

pub type TableEventHandler = Arc<Mutex<Box<dyn FnMut(Vec<String>, Vec<Field>) + Send>>>;
pub fn table_event_handler<F>(callback: F) -> TableEventHandler
where
    F: FnMut(Vec<String>, Vec<Field>) + Send + 'static,
{
    Arc::new(Mutex::new(Box::new(callback)))
}

#[derive(Default, Props)]
pub struct TableViewProps {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Field>>,
    pub has_focus: bool,
    pub on_add_row: Option<TableEventHandler>,
    pub on_update_row: Option<TableEventHandler>,
    pub on_delete_row: Option<TableEventHandler>,
}

fn field_to_string(field: &Field) -> String {
    match field {
        Field::Boolean(value) => value
            .boolean
            .map_or_else(String::new, |value| value.to_string()),
        Field::Str(value) => value.str.clone().unwrap_or_default(),
        Field::StrContainer(value) => value.strs.join(", "),
        Field::I8(value) => value.i8.map_or_else(String::new, |value| value.to_string()),
        Field::I16(value) => value
            .i16
            .map_or_else(String::new, |value| value.to_string()),
        Field::I32(value) => value
            .i32
            .map_or_else(String::new, |value| value.to_string()),
        Field::I64(value) => value
            .i64
            .map_or_else(String::new, |value| value.to_string()),
        Field::Datetime(value) => value
            .datetime
            .as_ref()
            .map_or_else(String::new, |value| format!("{value:?}")),
    }
}

#[component]
pub fn TableView(props: &TableViewProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut cursor = hooks.use_state(|| 0usize);
    let mut scroll_handle = hooks.use_ref_default::<ScrollViewHandle>();

    let rows = props.rows.clone();
    let mut rows_ref = hooks.use_ref(|| rows.clone());
    rows_ref.set(rows.clone());
    let columns = props.columns.clone();
    let mut columns_ref = hooks.use_ref(|| columns.clone());
    columns_ref.set(columns.clone());
    let row_count = rows.len();
    let mut row_count_ref = hooks.use_ref(|| row_count.clone());
    row_count_ref.set(row_count.clone());

    if rows.is_empty() {
        if cursor.get() != 0 {
            cursor.set(0);
        }
    } else if cursor.get() >= rows.len() {
        cursor.set(rows.len() - 1);
    }

    let cursor_index = cursor.get();
    let viewport_height = scroll_handle.read().viewport_height().max(0) as usize;
    let current_offset = scroll_handle.read().scroll_offset().max(0) as usize;
    if viewport_height > 0 {
        let next_offset = if cursor_index < current_offset {
            cursor_index
        } else if cursor_index >= current_offset + viewport_height {
            cursor_index - viewport_height + 1
        } else {
            current_offset
        };
        if next_offset != current_offset {
            scroll_handle.write().scroll_to(next_offset as i32);
        }
    }

    let has_focus = props.has_focus;

    let on_add_row = props.on_add_row.clone();
    let on_update_row = props.on_update_row.clone();
    let on_delete_row = props.on_delete_row.clone();

    hooks.use_hotkeys(
        move || has_focus,
        move |hotkey_manager| {
            let _ = hotkey_manager.register(
                &[(KeyCode::Char('k'), KeyModifiers::NONE, KeyEventKind::Press)],
                move |_: &mut ()| {
                    cursor.set(cursor.get().saturating_sub(1));
                    Ok(())
                },
            );
            let _ = hotkey_manager.register(
                &[(KeyCode::Char('j'), KeyModifiers::NONE, KeyEventKind::Press)],
                move |_: &mut ()| {
                    cursor.set((cursor.get() + 1).min(row_count_ref.get() - 1));
                    Ok(())
                },
            );
            let _ = hotkey_manager.register(
                &[
                    (KeyCode::Char('g'), KeyModifiers::NONE, KeyEventKind::Press),
                    (KeyCode::Char('g'), KeyModifiers::NONE, KeyEventKind::Press),
                ],
                move |_: &mut ()| {
                    cursor.set(0);
                    Ok(())
                },
            );
            let _ = hotkey_manager.register(
                &[(KeyCode::Char('G'), KeyModifiers::SHIFT, KeyEventKind::Press)],
                move |_: &mut ()| {
                    cursor.set(row_count_ref.get().saturating_sub(1));
                    Ok(())
                },
            );

            {
                let on_add_row = on_add_row.clone();
                let _ = hotkey_manager.register(
                    &[(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Press)],
                    move |_: &mut ()| {
                        if let Some(handler) = on_add_row.as_ref() {
                            if let Ok(mut handler) = handler.lock() {
                                let cur_pos = cursor.get();
                                let columns = columns_ref.read().clone();
                                let row =
                                    rows_ref.read().get(cur_pos).unwrap_or(&Vec::new()).clone();
                                handler(columns, row);
                            }
                        }
                        Ok(())
                    },
                );
            }

            {
                let on_update_row = on_update_row.clone();
                let _ = hotkey_manager.register(
                    &[(KeyCode::Char('i'), KeyModifiers::NONE, KeyEventKind::Press)],
                    move |_: &mut ()| {
                        if let Some(handler) = on_update_row.as_ref() {
                            if let Ok(mut handler) = handler.lock() {
                                let cur_pos = cursor.get();
                                let columns = columns_ref.read().clone();
                                let row =
                                    rows_ref.read().get(cur_pos).unwrap_or(&Vec::new()).clone();
                                handler(columns, row);
                            }
                        }
                        Ok(())
                    },
                );
            }

            {
                let on_delete_row = on_delete_row.clone();
                let _ = hotkey_manager.register(
                    &[
                        (KeyCode::Char('d'), KeyModifiers::NONE, KeyEventKind::Press),
                        (KeyCode::Char('d'), KeyModifiers::NONE, KeyEventKind::Press),
                    ],
                    move |_: &mut ()| {
                        if let Some(handler) = on_delete_row.as_ref() {
                            if let Ok(mut handler) = handler.lock() {
                                let cur_pos = cursor.get();
                                let columns = columns_ref.read().clone();
                                let row =
                                    rows_ref.read().get(cur_pos).unwrap_or(&Vec::new()).clone();
                                handler(columns, row);
                            }
                        }
                        Ok(())
                    },
                );
            }
        },
    );

    element! {
        View(width: 100pct, height: 100pct, flex_direction: FlexDirection::Column) {
            View(width: 100pct, flex_direction: FlexDirection::Row) {
                #(columns.iter().map(|column| element! {
                    View(width: 100pct, border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                        Text(content: column.clone())
                    }
                }))
            }
            ScrollView(handle: Some(scroll_handle), scrollbar: Some(true), keyboard_scroll: Some(false)) {
                View(width: 100pct, flex_direction: FlexDirection::Column) {
                    #(rows.iter().enumerate().map(|(index, row)| element! {
                        View(
                            width: 100pct,
                            flex_direction: FlexDirection::Row,
                            background_color: if index == cursor.get() { Some(Color::Blue) } else if index % 2 == 0 { Some(Color::DarkGrey) } else { None },
                        ) {
                            #(row.iter().map(|field| element! {
                                View(width: 100pct) {
                                    Text(content: field_to_string(field), color: if index == cursor.get() || index % 2 == 0 { Some(Color::Black) } else { None })
                                }
                            }))
                        }
                    }))
                }
            }
        }
    }
}
