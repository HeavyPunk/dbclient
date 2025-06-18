use std::{collections::HashMap, sync::{Arc, Mutex}};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Rect}, prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Row, Table}, Frame, Terminal};

use crate::{dbclient::fetcher::FetchResult, ui2::{pipe::Pipe, Widget}};

pub struct QueryResultWidget {
    pipe: Arc<Mutex<Pipe>>,
    list: Option<FetchResult>
}

impl QueryResultWidget {
    pub fn new(pipe: Arc<Mutex<Pipe>>) -> Self {
        Self {
            pipe,
            list: None,
        }
    }
}

impl<TerminalBackend> Widget<TerminalBackend> for QueryResultWidget
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut Frame, rect: &Rect, is_selected: bool) {       
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
                        data_rows.push(Row::new(row_data));
                    }
                    (headers, data_rows)
                },
                None => (vec!["None".to_string()], vec![Row::new(vec!["None"])])
            },
            None => (vec!["None".to_string()], vec![Row::new(vec!["None"])])
        };

        let widths = vec![Constraint::Fill(1); rows.0.len()];

        let query_result_block = Table::new(rows.1, widths)
            .header(Row::new(rows.0).style(Style::default().bg(Color::Blue).fg(Color::Black)))
            .block(Block::new().title("Result").borders(Borders::all()).style(style));
        frame.render_widget(query_result_block, *rect);
    }

    fn react_on_event(&mut self, _: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent) -> crate::ui2::WidgetReaction {
        match event {
            crate::ui2::UiEvent::None => crate::ui2::WidgetReaction::Nothing,
            crate::ui2::UiEvent::KeyboardEvent(key_event) => {
                match key_event {
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => crate::ui2::WidgetReaction::ExitFromWidget,
                    _ => crate::ui2::WidgetReaction::Nothing,
                }
            },
        }
    }
}

