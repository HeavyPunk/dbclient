use ratatui::{crossterm::event::{KeyCode, KeyEvent}, layout::Constraint, prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Row, Table}, Terminal};

use crate::{config::Connection, dbclient::{dummy::DummyFetcher, fetcher::Fetcher, redis::{RedisConfig, RedisFetcher}}, ui2::{pages::query::QueryPage, ui_mode::UserMode, Widget}};


pub struct ConnectionsListWidget {
    connections: Vec<Connection>,
    selected_connection_index: usize,
}

impl ConnectionsListWidget {
    pub fn new(connections: Vec<Connection>) -> Self {
        Self { 
            connections,
            selected_connection_index: 0
        }
    }
}

impl<TerminalBackend> Widget<TerminalBackend> for ConnectionsListWidget
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut ratatui::Frame, rect: &ratatui::prelude::Rect, _: &UserMode, is_selected: bool) {
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let rows: Vec<_> = self.connections.iter()
            .enumerate()
            .map(|(index, conn)| {
                let style = if index == self.selected_connection_index {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                Row::new(vec![conn.name.clone(), format!("{}", conn.connection_type)]).style(style)
            })
            .collect();
        let connection_table = Table::new(rows, &[Constraint::Fill(3), Constraint::Fill(1)])
            .block(Block::new().borders(Borders::all()).title("Available Connections").style(style));
        frame.render_widget(connection_table, *rect);
    }

    fn react_on_event(&mut self, terminal: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent, _: &UserMode) -> crate::ui2::WidgetReaction {
        match event {
            crate::ui2::UiEvent::KeyboardEvent(key_event) => {
                match key_event {
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => crate::ui2::WidgetReaction::ExitFromWidget,
                    KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => {
                        // TODO: enter to query page
                        let connection = self.connections.get(self.selected_connection_index).unwrap();
                        match connection.connection_type {
                            crate::config::ConnectionType::Redis => {
                                let fetcher = RedisFetcher {
                                    config: RedisConfig { uri: connection.connection_string.clone() }
                                };
                                run_query_page(terminal, fetcher);
                            },
                            crate::config::ConnectionType::Postgres => {
                                let fetcher = DummyFetcher::new();
                                run_query_page(terminal, fetcher);
                            },
                            crate::config::ConnectionType::MySql => {
                                let fetcher = DummyFetcher::new();
                                run_query_page(terminal, fetcher);
                            },
                        };
                        // let query_page = QueryPage::new(terminal, fetcher)
                        crate::ui2::WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ } => {
                        if self.selected_connection_index < self.connections.len() - 1 {
                            self.selected_connection_index += 1;
                        }
                        crate::ui2::WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ } => {
                        if self.selected_connection_index > 0 {
                            self.selected_connection_index -= 1;
                        }
                        crate::ui2::WidgetReaction::Nothing
                    }
                    _ => crate::ui2::WidgetReaction::Nothing,
                }
            },
        }
    }
}

fn run_query_page(terminal: &mut Terminal<impl Backend>, fetcher: impl Fetcher + 'static) {
    let mut query_page = QueryPage::new(terminal, fetcher);
    query_page.render(terminal);
    query_page.run_event_loop(terminal);
}

