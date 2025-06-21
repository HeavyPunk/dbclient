use std::sync::{Arc, Mutex};

use ratatui::{crossterm::event::{Event, KeyCode, KeyEvent}, layout::Rect, prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Paragraph, Wrap}, Frame, Terminal};
use string_cmd::{events::event_to_command, StringEditor};

use crate::{dbclient::{fetcher::{FetchRequest, Fetcher}, query_builder::QueryElement}, ui2::{pipe::{Payload, Pipe}, ui_mode::UserMode, UiEvent, Widget, WidgetReaction}};

pub struct QueryLineWidget<Client>
where
    Client: Fetcher,
{
    fetcher: Arc<Mutex<Client>>,
    pipe: Arc<Mutex<Pipe>>,
    query: StringEditor,
    cmd_string: Option<StringEditor>,
    cursor_pos: Option<Rect>,
    edit_mode: bool,
}

impl<Client> QueryLineWidget<Client>
where
    Client: Fetcher,
{
    pub fn new(fetcher: Arc<Mutex<Client>>, pipe: Arc<Mutex<Pipe>>) -> Self {
        Self { fetcher, pipe, query: StringEditor::new(), cmd_string: None, cursor_pos: None, edit_mode: false }
    }
}

impl<Client, TerminalBackend> Widget<TerminalBackend> for QueryLineWidget<Client>
where
    Client: Fetcher,
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut Frame, rect: &Rect, _: &UserMode, is_selected: bool) {
        let style = if is_selected {
            let cursor = self.query.cursor_pos();
            let rend_curs = (rect.x + 1 + cursor as u16 % rect.width, rect.y + 1 + cursor as u16 / rect.width);
            frame.set_cursor_position(rend_curs);
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let query_line_block = Paragraph::new(self.query.get_text())
            .wrap(Wrap { trim: true })
            .block(Block::new().title("Query").borders(Borders::all()).style(style));
        frame.render_widget(query_line_block, *rect);
    }

    fn react_on_event(&mut self, _: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent, user_mode: &UserMode) -> crate::ui2::WidgetReaction {
        match user_mode {
            UserMode::Normal => {
                match event {
                    UiEvent::KeyboardEvent(key_event) => {
                        match key_event {
                            KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => {
                                self.cursor_pos = None;
                                if self.edit_mode { self.edit_mode = false; WidgetReaction::Nothing } else { WidgetReaction::ExitFromWidget }
                            },
                            KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => {
                                if let Some(cmd_string) = self.cmd_string.as_mut() {
                                    match cmd_string.get_text() {
                                        "w" => {
                                            let mut fetcher = self.fetcher.lock().unwrap();
                                            let req = FetchRequest {
                                                query: vec![QueryElement::RawQuery(self.query.get_text().to_string())],
                                                limit: 100
                                            };
                                            match fetcher.fetch(&req) {
                                                Ok(res) => {
                                                    let mut pipe = self.pipe.lock().unwrap();
                                                    let _ = pipe.push_message(Payload::DbObjects(res));
                                                },
                                                Err(_) => (),
                                            };
                                        },
                                        _ => ()
                                    };
                                    self.cmd_string = None;
                                }
                                WidgetReaction::Nothing
                            },
                            _ => WidgetReaction::Nothing
                        }
                    }
                }
            },
            UserMode::Insert => {
                match event {
                    UiEvent::KeyboardEvent(key_event) => {
                        if !self.edit_mode {
                            self.edit_mode = true;
                        } else {
                            if let Some(command) = event_to_command(&Event::Key(key_event.clone())) {
                                self.query.execute(command);
                            }
                        }
                        WidgetReaction::Nothing
                    }
                }
            },
            UserMode::SearchInput => todo!(),
            UserMode::Search(_, _) => todo!(),
            UserMode::Command => {
                match event {
                    UiEvent::KeyboardEvent(key_event) => {
                        match key_event {
                            KeyEvent { code: KeyCode::Char(':'), modifiers: _, kind: _, state: _ } => self.cmd_string = Some(StringEditor::new()),
                            _ => {
                                if let Some(cmd_string) = self.cmd_string.as_mut() {
                                    if let Some(command) = event_to_command(&Event::Key(key_event.clone())) {
                                        cmd_string.execute(command);
                                        let mut pipe = self.pipe.lock().unwrap();
                                        pipe.push_message(Payload::UserMode(cmd_string.get_text().to_string())).unwrap();
                                    }
                                }
                            }
                        };
                        WidgetReaction::Nothing
                    }
                }
            },
        }
    }
}

