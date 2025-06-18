use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Paragraph}, Frame, Terminal};

use crate::ui2::{pipe::Pipe, Widget};

pub struct QueryLineWidget {
    pipe: Arc<Mutex<Pipe>>
}

impl QueryLineWidget {
    pub fn new(pipe: Arc<Mutex<Pipe>>) -> Self {
        Self { pipe }
    }
}

impl<TerminalBackend> Widget<TerminalBackend> for QueryLineWidget
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut Frame, rect: &Rect, is_selected: bool) {
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let query_line_block = Paragraph::new("Lalala")
            .block(Block::new().title("Query").borders(Borders::all()).style(style));
        frame.render_widget(query_line_block, *rect);
    }

    fn react_on_event(&mut self, _: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent) -> crate::ui2::WidgetReaction {
        match event {
            crate::ui2::UiEvent::KeyboardEvent(key_event) => {
                match key_event {
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => crate::ui2::WidgetReaction::ExitFromWidget,
                    _ => crate::ui2::WidgetReaction::Nothing,
                }
            },
        }
    }
}

