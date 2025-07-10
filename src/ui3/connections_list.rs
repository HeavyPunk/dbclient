use ratatui::{layout::Alignment, style::Color};
use tuirealm::{event::{Key, KeyEvent}, props::{BorderType, Borders, Table, TableBuilder, TextSpan}, Component, Event, MockComponent};

use crate::config::Connection;

use super::{AppEvent, Msg};

#[derive(MockComponent)]
pub struct ConnectionsListComponent {
    component: tui_realm_stdlib::Table
}

impl Default for ConnectionsListComponent {
    fn default() -> Self {
        Self {
            component: tui_realm_stdlib::Table::default()
                .title("Available connections", Alignment::Left)
                .highlighted_color(Color::LightYellow)
                .highlighted_str("> ")
                .scroll(true)
                .rewind(true)
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Yellow)
                )
        }
    }
}

impl Component<Msg, AppEvent> for ConnectionsListComponent {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::AppClose),
            Event::Keyboard(KeyEvent { code: Key::Char('j') | Key::Down, .. }) => {
                self.component.states.incr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('k') | Key::Up, .. }) => {
                self.component.states.decr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Enter, .. }) => {
                let selected_connection = self.component.states.list_index;
                Some(Msg::ToQueryPage(selected_connection))
            },
            _ => Some(Msg::None),
        }
    }
}

impl ConnectionsListComponent {
    pub fn build_connections_table(connections: &Vec<Connection>) -> Table {
        if connections.is_empty() {
            return vec![];
        }
        let mut table = TableBuilder::default();
        connections.iter().enumerate().for_each(|(index, conn)| {
            let row = table
                .add_col(TextSpan::from(conn.name.clone()).fg(Color::Blue))
                .add_col(TextSpan::from(conn.connection_type.to_string()));
            if index < connections.len() - 1 {
                row.add_row();
            }
        });
        table.build()
    }
}

