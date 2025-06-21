use std::sync::{Arc, Mutex};

use ratatui::{crossterm::event::{KeyCode, KeyEvent}, layout::{Constraint, Rect}, prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Row, Table, TableState}, Frame, Terminal};

use crate::{dbclient::fetcher::FetchResult, ui2::{pipe::Pipe, ui_mode::UserMode, UiEvent, Widget, WidgetReaction}};

pub struct QueryResultWidget {
    pipe: Arc<Mutex<Pipe>>,
    list: Option<FetchResult>,
    selected_line_index: usize,
}

impl QueryResultWidget {
    pub fn new(pipe: Arc<Mutex<Pipe>>) -> Self {
        Self {
            pipe,
            list: None,
            selected_line_index: 0,
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
                Some(columns) => {
                    let headers: Vec<String> = columns.keys().cloned().collect();
                    let mut data_rows: Vec<Row> = vec![];
                    
                    let max_len = columns.values().map(|v| v.len()).max().unwrap_or(0);
                    for i in 0..max_len {
                        let row_data: Vec<String> = columns.values()
                            .map(|column| column.get(i).cloned().unwrap_or_else(|| "".to_string()))
                            .collect();
                        let row_style = if is_selected && i == self.selected_line_index {
                            Style::default().bg(Color::Yellow).fg(Color::Black)
                        } else {
                            Style::default()
                        };
                        data_rows.push(Row::new(row_data).style(row_style));
                    }
                    (headers, data_rows)
                },
                None => (vec!["None".to_string()], vec![Row::new(vec!["None"])])
            },
            None => (vec!["None".to_string()], vec![Row::new(vec!["None"])])
        };

        let widths = vec![Constraint::Fill(1); rows.0.len()];

        let mut table_state = TableState::default();
        table_state.select(Some(self.selected_line_index));

        let query_result_block = Table::new(rows.1, widths)
            .header(Row::new(rows.0).style(Style::default().bg(Color::Blue).fg(Color::Black)))
            .block(Block::new().title("Result").borders(Borders::all()).style(style));
        // frame.render_widget(query_result_block, *rect);
        frame.render_stateful_widget(query_result_block, *rect, &mut table_state);
    }

    fn react_on_event(&mut self, _: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent, user_mode: &UserMode) -> crate::ui2::WidgetReaction {
        match user_mode {
            UserMode::Normal => match event {
                UiEvent::KeyboardEvent(key_event) => match key_event {
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => WidgetReaction::ExitFromWidget,
                    KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ } => {
                        if let Some(list) = &self.list {
                            if self.selected_line_index < list.get_table_height() - 1 {
                                self.selected_line_index += 1;
                            }
                        }
                        WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ } => {
                        if self.selected_line_index > 0 {
                            self.selected_line_index -= 1;
                        }
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

