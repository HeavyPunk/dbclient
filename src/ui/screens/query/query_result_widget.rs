use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Rect}, style::{Color, Style}, widgets::{Block, Borders, Paragraph, Row, Table}};

use crate::ui::screens::Widget;

pub struct QueryResultWidgetState {

}

pub struct QueryResultWidget<'a> {
    pub state: &'a mut QueryResultWidgetState,
    pub place: &'a Rect
}

impl<'a> QueryResultWidget<'a> {
    pub fn new(state: &'a mut QueryResultWidgetState, place: &'a Rect) -> Self {
        Self { state, place }
    }
}

impl<'a> Widget for QueryResultWidget<'a> {
    fn draw(&mut self, frame: &mut ratatui::Frame, is_selected: bool) {
        // let query_result_block = Table::new(vec![Row::new(vec!["None"])], [Constraint::Fill(1)]);
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let query_result_block = Paragraph::new("None")
            .block(Block::new().title("Result").borders(Borders::all()).style(style));
        frame.render_widget(query_result_block, *self.place);
    }

    fn handle_events(&mut self, terminal: &mut ratatui::Terminal<impl ratatui::prelude::Backend>) {
        loop {
            match event::read().expect("failed to read events") {
                event::Event::Key(key_event) => {
                    match key_event {
                        KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => break,
                        _ => ()
                    }
                },
                _ => ()
            }
            terminal.draw(|frame| self.draw(frame, true)).expect("failed to render");
        }
    }

    fn get_content(&mut self) -> crate::ui::screens::Content {
        crate::ui::screens::Content::SimpleString(String::from(""))
    }
}

