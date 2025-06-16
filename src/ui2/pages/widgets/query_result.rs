use std::sync::{Arc, Mutex};

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
            Some(r) => match &r.rows {
                Some(rows) => rows.iter().map(|row| Row::new(row.columns.clone())).collect(),
                None => vec![Row::new(vec!["None"])],
            },
            None => vec![Row::new(vec!["None"])],
        };

        let widths = vec![Constraint::Fill(1); rows.len()];

        let query_result_block = Table::new(rows, widths)
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

