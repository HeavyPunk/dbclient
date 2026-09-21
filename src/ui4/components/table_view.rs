use std::sync::{Arc, Mutex};

use iocraft::{
    component,
    components::{BorderStyle, ScrollView, ScrollViewHandle, Text, View},
    element,
    hooks::{UseRef, UseState, UseTerminalEvents},
    AnyElement, Color, Edges, FlexDirection, Hooks, KeyCode, KeyEvent, KeyEventKind, Props,
    TerminalEvent,
};
use tokio::sync::Mutex as TokioMutex;

use crate::core::proto::common::db_field::Field;

pub type TableKeyHandler = Arc<Mutex<Box<dyn FnMut(KeyCode, (Vec<String>, Vec<Field>)) + Send>>>;
pub fn table_key_handler<F>(callback: F) -> TableKeyHandler
    where F: FnMut(KeyCode, (Vec<String>, Vec<Field>)) + Send + 'static
{
    Arc::new(Mutex::new(Box::new(callback)))
}

#[derive(Default, Props)]
pub struct TableViewProps {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Field>>,
    pub has_focus: bool,
    pub selected_row: Arc<TokioMutex<Option<usize>>>,
    pub on_key: Option<TableKeyHandler>,
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
    let columns = props.columns.clone();
    let row_count = rows.len();

    if rows.is_empty() {
        cursor.set(0);
    } else if cursor.get() >= rows.len() {
        cursor.set(rows.len() - 1);
    }

    let current_row = if rows.is_empty() {
        None
    } else {
        Some(cursor.get())
    };
    tokio::task::block_in_place(|| *props.selected_row.blocking_lock() = current_row);

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

    let on_key = props.on_key.clone();
    let has_focus = props.has_focus;
    let keybind_columns = columns.clone();
    let keybind_rows = rows.clone();
    hooks.use_local_terminal_events(move |event| {
        if !has_focus {
            return;
        }
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }

        if let Some(handler) = on_key.as_ref() {
            if let Ok(mut handler) = handler.lock() {
                let cur_pos = cursor.get();
                let columns = keybind_columns.clone();
                let row = keybind_rows.get(cur_pos).unwrap_or(&Vec::new()).clone();
                handler(code, (columns, row));
            }
        }

        match code {
            KeyCode::Down | KeyCode::Char('j') if row_count > 0 => {
                cursor.set((cursor.get() + 1).min(row_count - 1));
            }
            KeyCode::Up | KeyCode::Char('k') if row_count > 0 => {
                cursor.set(cursor.get().saturating_sub(1));
            }
            _ => {}
        }
    });

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
                                    Text(content: field_to_string(field), color: if index == cursor.get() { Some(Color::Black) } else { None })
                                }
                            }))
                        }
                    }))
                }
            }
        }
    }
}
