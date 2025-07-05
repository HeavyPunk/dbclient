use ratatui::{layout::Alignment, style::Color};
use tuirealm::{application::ApplicationResult, command::{Cmd, Direction, Position}, event::{Key, KeyEvent}, props::{BorderType, Borders, InputType}, Component, Event, MockComponent};

use super::{AppEvent, Msg};

enum InputMode {
    Input,
    Normal,
}

#[derive(MockComponent)]
pub struct QueryInput {
    component: tui_realm_stdlib::Input,
    input_mode: InputMode,
}

impl Default for QueryInput {
    fn default() -> Self {
        let input = tui_realm_stdlib::Input::default()
            .title("Query", Alignment::Left)
            .input_type(InputType::Text)
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Yellow)
            );
        Self {
            component: input,
            input_mode: InputMode::Normal,
        }
    }
}

impl Component<Msg, AppEvent> for QueryInput {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
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
                        let query = self.component.states.get_value();
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

