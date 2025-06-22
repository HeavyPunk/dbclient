use std::sync::{Arc, Mutex};

use ratatui::{crossterm::event::{KeyCode, KeyEvent}, layout::{Constraint, Rect}, prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Cell, Row, Table, TableState}, Frame, Terminal};

use crate::{dbclient::fetcher::FetchResult, ui2::{pipe::Pipe, ui_mode::UserMode, UiEvent, Widget, WidgetReaction}};

pub struct QueryResultWidget {
    pipe: Arc<Mutex<Pipe>>,
    list: Option<FetchResult>,
    selected_cell_index: (usize, usize),
}

impl QueryResultWidget {
    pub fn new(pipe: Arc<Mutex<Pipe>>) -> Self {
        Self {
            pipe,
            list: None,
            selected_cell_index: (0, 0),
        }
    }
}

impl<TerminalBackend> Widget<TerminalBackend> for QueryResultWidget
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut Frame, rect: &Rect, _: &UserMode, is_selected: bool) {       
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let mut pipe = self.pipe.lock().unwrap();
        match pipe.try_get_db_objects() {
            Ok(res) => self.list = Some(res),
            Err(_) => (),
        };

        let rows = match &self.list {
            Some(r) => match &r.table {
                Some(table) => {
                    let headers: Vec<String> = table.1.keys().cloned().collect();
                    let mut data_rows: Vec<Row> = vec![];

                    let max_len = table.1.values().map(|v| v.len()).max().unwrap_or(0);
                    for row_index in 0..max_len {
                        let mut row_data: Vec<Cell> = vec![];
                        for (column_index, column) in table.1.values().enumerate() {
                            let style = if self.selected_cell_index.0 == column_index && self.selected_cell_index.1 == row_index {
                                Style::default().bg(Color::Yellow).fg(Color::Black)
                            } else {
                                Style::default()
                            };
                            let val = column.get(row_index).cloned().unwrap_or_else(|| "".to_string());
                            row_data.push(Cell::new(val).style(style));
                        }
                        data_rows.push(Row::new(row_data));
                    }
                    (headers, data_rows)
                },
                None => (vec!["None".to_string()], vec![Row::new(vec!["None"])])
            },
            None => (vec!["None".to_string()], vec![Row::new(vec!["None"])])
        };

        let widths = vec![Constraint::Fill(1); rows.0.len()];

        let mut table_state = TableState::default();
        table_state.select(Some(self.selected_cell_index.1));

        let query_result_block = Table::new(rows.1, widths)
            .header(Row::new(rows.0).style(Style::default().bg(Color::Blue).fg(Color::Black)))
            .block(Block::new().title("Result").borders(Borders::all()).style(style));
        frame.render_stateful_widget(query_result_block, *rect, &mut table_state);
    }

    fn react_on_event(&mut self, _: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent, user_mode: &UserMode) -> crate::ui2::WidgetReaction {
        match user_mode {
            UserMode::Normal => match event {
                UiEvent::KeyboardEvent(key_event) => match key_event {
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => WidgetReaction::Nothing,
                    KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ } => {
                        if let Some(list) = &self.list {
                            if self.selected_cell_index.1 < list.get_table_height() - 1 {
                                self.selected_cell_index.1 += 1;
                            }
                        }
                        WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ } => {
                        if self.selected_cell_index.1 > 0 {
                            self.selected_cell_index.1 -= 1;
                        }
                        WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('h'), modifiers: _, kind: _, state: _ } => {
                        self.selected_cell_index.0 = if self.selected_cell_index.0 > 0 { self.selected_cell_index.0 - 1 } else { self.selected_cell_index.0 };
                        WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('l'), modifiers: _, kind: _, state: _ } => {
                        self.selected_cell_index.0 = if self.selected_cell_index.0 < self.list.as_ref().unwrap_or(&FetchResult::none()).get_table_width() - 1 { self.selected_cell_index.0 + 1 } else { self.selected_cell_index.0 };
                        WidgetReaction::Nothing
                    }
                    _ => WidgetReaction::Nothing,
                }
            },
            UserMode::Insert => todo!(),
            UserMode::SearchInput => todo!(),
            UserMode::Search(_, _) => todo!(),
            UserMode::Command => todo!(),
        }
    }
}

