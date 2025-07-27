use ratatui::{layout::Alignment, style::{Color, Modifier, Style}, widgets::Clear};
use tui_realm_textarea::{TextArea, TEXTAREA_CMD_NEWLINE, TEXTAREA_STATUS_FMT};
use tuirealm::{command::{Cmd, CmdResult, Direction, Position}, event::{Key, KeyEvent}, props::{BorderType, Borders, PropPayload, PropValue}, AttrValue, Attribute, Component, Event, MockComponent};

use super::{AppEvent, Msg, editor_popup::EditorPopupWidget};

#[derive(Clone)]
enum InputMode {
    Input,
    Normal,
}

impl Into<String> for InputMode {
    fn into(self) -> String {
        match self {
            InputMode::Input => "INPUT".to_string(),
            InputMode::Normal => "NORMAL".to_string(),
        }
    }
}

impl Into<Style> for InputMode {
    fn into(self) -> Style {
        match self {
            InputMode::Input => Style::default().fg(Color::Black).bg(Color::Green),
            InputMode::Normal => Style::default().fg(Color::Black).bg(Color::Blue),
        }
    }
}


pub struct EditorInput {
    component: TextArea<'static>,
    input_mode: InputMode,
    pub editor_type: &'static str,
}

impl EditorInput {
    pub fn new(title: &'static str, editor_type: &'static str) -> Self {
        Self {
            component: TextArea::default()
                .title(title, Alignment::Left)
                .layout_margin(0)
                .scroll_step(1)
                .cursor_line_style(Style::default())
                .cursor_style(Style::default().add_modifier(Modifier::REVERSED))
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Yellow)
                )
            ,
            input_mode: InputMode::Input,
            editor_type
        }
    }
}

impl Component<Msg, AppEvent> for EditorInput {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        self.component.attr(
            Attribute::Custom(TEXTAREA_STATUS_FMT),
            AttrValue::Payload(PropPayload::Tup2((
                PropValue::Str(self.input_mode.clone().into()),
                PropValue::Style(self.input_mode.clone().into()),
            ))),
        );

        match self.input_mode {
            InputMode::Input => {
                match ev {
                    Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                        self.input_mode = InputMode::Normal;
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char(ch), .. }) => {
                        self.component.perform(Cmd::Type(ch));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Left, .. }) => {
                        self.component.perform(Cmd::Move(Direction::Left));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Right, .. }) => {
                        self.component.perform(Cmd::Move(Direction::Right));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Down, .. }) => {
                        self.component.perform(Cmd::Move(Direction::Down));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                        self.component.perform(Cmd::Move(Direction::Up));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Backspace, .. }) => {
                        self.component.perform(Cmd::Delete);
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Delete, .. }) => {
                        self.component.perform(Cmd::Cancel);
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Enter, .. }) => {
                        self.component.perform(Cmd::Custom(TEXTAREA_CMD_NEWLINE));
                        Some(Msg::None)
                    }
                    _ => Some(Msg::None)
                }
            },
            InputMode::Normal => {
                match ev {
                    Event::Keyboard(KeyEvent { code: Key::Enter, ..}) => Some(Msg::EditorAccept),
                    Event::Keyboard(KeyEvent { code: Key::Tab, ..}) => Some(Msg::EditorPopupNext),
                
                    Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::DiactivateEditor),
                    Event::Keyboard(KeyEvent { code: Key::Char('i'), .. }) => {
                        self.input_mode = InputMode::Input;
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('a'), .. }) => {
                        self.input_mode = InputMode::Input;
                        self.component.perform(Cmd::Move(Direction::Right));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('I'), .. }) => {
                        self.input_mode = InputMode::Input;
                        self.component.perform(Cmd::GoTo(Position::Begin));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('A'), .. }) => {
                        self.input_mode = InputMode::Input;
                        self.component.perform(Cmd::GoTo(Position::End));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('h'), .. }) => {
                        self.component.perform(Cmd::Move(Direction::Left));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('l'), .. }) => {
                        self.component.perform(Cmd::Move(Direction::Right));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('j'), .. }) => {
                        self.component.perform(Cmd::Move(Direction::Down));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('k'), .. }) => {
                        self.component.perform(Cmd::Move(Direction::Up));
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Char('o'), .. }) => {
                        self.component.perform(Cmd::Custom(TEXTAREA_CMD_NEWLINE));
                        Some(Msg::None)
                    },
                    _ => Some(Msg::None)
                }
            },
        }
    }
}

impl MockComponent for EditorInput {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        frame.render_widget(Clear, area);
        self.component.view(frame, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    fn state(&self) -> tuirealm::State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl EditorPopupWidget for EditorInput {
    fn get_content(&self) -> Vec<String> {
        match self.component.state() {
            tuirealm::State::One(text_val) => {
                text_val.unwrap_string()
                    .lines()
                    .map(|s| s.to_string())
                    .collect()
            },
            tuirealm::State::Vec(lines) => {
                lines.iter()
                    .map(|v| v.clone().unwrap_string())
                    .collect()
            },
            _ => vec![]
        }
    }
    
    fn get_editor_type(&self) -> &'static str {
        self.editor_type
    }
}

