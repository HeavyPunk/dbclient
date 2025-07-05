use ratatui::{layout::Alignment, style::{Color, Modifier, Style}};
use tui_realm_textarea::{TextArea, TEXTAREA_STATUS_FMT};
use tuirealm::{command::{Cmd, CmdResult, Direction, Position}, event::{Key, KeyEvent}, props::{BorderType, Borders, PropPayload, PropValue}, AttrValue, Attribute, Component, Event, MockComponent};

use super::{AppEvent, Msg};

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

#[derive(MockComponent)]
pub struct QueryInput {
    component: TextArea<'static>,
    input_mode: InputMode,
}

impl Default for QueryInput {
    fn default() -> Self {
        let text_area = TextArea::default()
            .title("Query", Alignment::Left)
            .layout_margin(0)
            .cursor_line_style(Style::default())
            .cursor_style(Style::default().add_modifier(Modifier::REVERSED))
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Yellow)
            );
        Self {
            component: text_area,
            input_mode: InputMode::Normal,
        }
    }
}

impl Component<Msg, AppEvent> for QueryInput {
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
                    Event::Keyboard(KeyEvent { code: Key::Backspace, .. }) => {
                        self.component.perform(Cmd::Delete);
                        Some(Msg::None)
                    },
                    Event::Keyboard(KeyEvent { code: Key::Delete, .. }) => {
                        self.component.perform(Cmd::Cancel);
                        Some(Msg::None)
                    },
                    _ => Some(Msg::None)
                }
            },
            InputMode::Normal => {
                match ev {
                    Event::Keyboard(KeyEvent { code: Key::Enter, ..}) => {
                        let query = match self.component.perform(Cmd::Submit) {
                            CmdResult::Submit(state) => {
                                let state_lines: Vec<String> = state.unwrap_vec().iter().map(|state_val| state_val.clone().unwrap_string()).collect();
                                state_lines.concat()
                            },
                            _ => "".to_string()
                        };
                        Some(Msg::ExecuteCustomQuery(query))
                    }
                    Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
                    Event::Keyboard(KeyEvent { code: Key::Char('J'), ..}) => Some(Msg::ToQueryResultWidget),
                    Event::Keyboard(KeyEvent { code: Key::Char('H'), ..}) => Some(Msg::ToDbObjectsWidget),
                    Event::Keyboard(KeyEvent { code: Key::Char('i'), .. }) => {
                        self.input_mode = InputMode::Input;
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
                    _ => Some(Msg::None)
                }
            },
        }
    }
}

