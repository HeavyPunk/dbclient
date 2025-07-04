use ratatui::{layout::Alignment, style::Color};
use tui_realm_stdlib::{List};
use tuirealm::{event::{Key, KeyEvent}, props::{BorderType, Borders, Table, TableBuilder, TextSpan}, AttrValue, Attribute, Component, Event, MockComponent};

use super::{AppEvent, Msg};

#[derive(MockComponent)]
pub struct DbObjects {
    component: tui_realm_stdlib::List,
}

impl Default for DbObjects {
    fn default() -> Self {
        let list = List::default()
            .title("Available Objects", Alignment::Left)
            .highlighted_color(Color::Yellow)
            .highlighted_str("> ")
            .scroll(true)
            .rewind(true)
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Yellow)
            );

        Self {
            component: list
        }
    }
}

impl Component<Msg, AppEvent> for DbObjects {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
            Event::Keyboard(KeyEvent { code: Key::Char('r'), .. }) => Some(Msg::FetchDbObjects),
            Event::Keyboard(KeyEvent { code: Key::Char('j'), .. }) => {
                self.component.states.incr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('k'), .. }) => {
                self.component.states.decr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('L'), ..}) => Some(Msg::ToQueryResultWidget),
            Event::Keyboard(KeyEvent { code: Key::Enter, .. }) => {
                match self.component.query(Attribute::Content) {
                    Some(val) => match val {
                        AttrValue::Table(list) => {
                            let current_object = list.get(self.component.states.list_index).unwrap().get(0).unwrap();
                            Some(Msg::FetchDbObject(current_object.content.clone()))
                        },
                        _ => Some(Msg::None)
                    },
                    None => Some(Msg::None),
                }
            },
            _ => Some(Msg::None)
        }
    }
}

impl DbObjects {
    pub fn build_objects_list(connections: &Vec<String>) -> Table {
        if connections.is_empty() {
            return vec![];
        }
        let mut table = TableBuilder::default();
        connections.iter().enumerate().for_each(|(index, obj)| {
            let row = table
                .add_col(TextSpan::from(obj).fg(Color::Blue));
            if index < connections.len() - 1 {
                row.add_row();
            }
        });
        table.build()
    }
}

