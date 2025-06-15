use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, style::{Color, Style}, widgets::{Block, Borders, Paragraph}, Frame};

use crate::ui2::Widget;

pub struct QueryResultWidget {

}

impl Widget for QueryResultWidget {
    fn render(&mut self, frame: &mut Frame, rect: &Rect, is_selected: bool) {       
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let query_result_block = Paragraph::new("None")
            .block(Block::new().title("Result").borders(Borders::all()).style(style));
        frame.render_widget(query_result_block, *rect);
    }

    fn react_on_event(&mut self, event: crate::ui2::UiEvent) -> crate::ui2::WidgetReaction {
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

