use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{layout::Rect, style::{Color, Style}, widgets::{Block, Borders, Paragraph}};
use crate::ui::{command, editor, screens::{Content, InputMode, Widget}};


pub struct QueryWidgetState {
    pub input_mode: InputMode,
    pub query_string: String,
    pub command_string: String,
}

pub struct QueryWidget<'a> {
    pub state: &'a mut QueryWidgetState,
    pub place: &'a Rect,
}

impl<'a> QueryWidget<'a> {
    pub fn new(state: &'a mut QueryWidgetState, place: &'a Rect) -> Self {
        Self { state, place }
    }
}

impl<'a> Widget for QueryWidget<'a> {
    fn draw(&mut self, frame: &mut ratatui::Frame, is_selected: bool) {
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let query_line_block = Paragraph::new(self.state.query_string.clone())
            .block(Block::new().title("Query").borders(Borders::all()).style(style));
        frame.render_widget(query_line_block, *self.place);
    }

    fn handle_events(&mut self, terminal: &mut ratatui::Terminal<impl ratatui::prelude::Backend>) {
        loop {
            match event::read().expect("failed to read events") {
                event::Event::Key(key_event) => {
                    match key_event {
                        KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => break,
                        KeyEvent { code: KeyCode::Char('w'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Press, state: _ } => break,
                        KeyEvent { code: KeyCode::Char('i'), modifiers: _, kind: _, state: _ } => {
                            self.state.input_mode = InputMode::Editing;
                            editor::edit(self.state.query_string.clone(), |new_val| {
                                self.state.query_string = new_val;
                                terminal.draw(|frame| self.draw(frame, true)).expect("failed to render");
                            });
                        },
                        KeyEvent { code: KeyCode::Char(':'), modifiers: _, kind: _, state: _ } => command::command(|new_val, kind| {
                            match kind {
                                command::EventKind::Edit => self.state.command_string = new_val,
                                command::EventKind::Enter => self.state.command_string = String::from(""),
                            };
                            terminal.draw(|frame| self.draw(frame, true)).expect("failed to render");
                        }),
                        _ => ()
                    }
                },
                _ => ()
            };
        }
        terminal.draw(|frame| self.draw(frame, true)).expect("failed to render");
    }

    fn get_content(&mut self) -> Content {
        Content::SimpleString(self.state.query_string.clone())
    }
}

