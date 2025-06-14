use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::prelude::Backend;
use ratatui::widgets::{Block, Row, Table};
use ratatui::Terminal;
use ratatui::{layout::Layout, Frame};
use ratatui::layout::Constraint::{Length, Min};
use crate::config::Connection;
use crate::dbclient::redis::{RedisConfig, RedisFetcher};
use super::query::{QueryPage, QueryState, QueryWidgetSelector};
use super::{InputMode, Screen};

pub struct MainState<'a> {
    pub connections_list: &'a Vec<Connection>,
    pub selected_connection_index: usize,
}

impl<'a> MainState<'a> {
    pub fn select_next_connection(&mut self) {
        let max_index = self.connections_list.len() - 1;
        if self.selected_connection_index < max_index {
            self.selected_connection_index += 1;
        }
    }

    pub fn select_prev_connection(&mut self) {
        if self.selected_connection_index > 0 {
            self.selected_connection_index -= 1;
        }
    }
}

pub struct MainPage<'a> {
    state: &'a mut MainState<'a>,
}

impl<'a> MainPage<'a> {
    pub fn new(state: &'a mut MainState<'a>) -> MainPage<'a> {
        MainPage { state }
    }
}

impl<'a> Screen for MainPage<'a> {
    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Min(0)]);
        let [main_area] = vertical.areas(frame.area());
        let rows: Vec<Row> = self.state.connections_list
            .iter()
            .enumerate()
            .map(|(index, connection)| {
                let style = if index == self.state.selected_connection_index {
                    ratatui::style::Style::default().bg(ratatui::style::Color::Yellow).fg(ratatui::style::Color::Black)
                } else {
                    ratatui::style::Style::default()
                };
                Row::new(vec![connection.name.clone(), format!("{}", connection.connection_type)]).style(style)
            })
            .collect();

        let connection_table = Table::new(rows, &[Length(30), Length(20)])
            .block(Block::bordered().title("Available Connections"));
        frame.render_widget(connection_table, main_area);
    }

    fn handle_events(&mut self, terminal: &mut Terminal<impl Backend>) {
        loop {
            match event::read().expect("failed to read event") {
                event::Event::Key(e) => {
                    if matches!(e, KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }) {

                        let connection = self.state.connections_list.get(self.state.selected_connection_index).unwrap();
                        match connection.connection_type {
                            crate::config::ConnectionType::Redis => {
                                let client = RedisFetcher {
                                    config: RedisConfig {
                                        uri: connection.connection_string.clone()
                                    },
                                };
                                let mut state = QueryState {
                                    connection: connection,
                                    client: client,
                                    query_string: String::from("KEYS *"),
                                    query_widget: QueryWidgetSelector::QueryWidget,
                                };
                                let mut query_page = QueryPage::new(&mut state);
                                terminal.draw(|frame| query_page.draw(frame)).expect("failed to render query page");
                                query_page.handle_events(terminal);
                            },
                            crate::config::ConnectionType::Postgres => unimplemented!(),
                            crate::config::ConnectionType::MySql => unimplemented!(),
                        };
                    }

                    if matches!(e, KeyEvent { code: KeyCode::Char('k'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }) {
                        // self.state.select_next_connection();
                        self.state.select_prev_connection();
                    }

                    if matches!(e, KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }) {
                        // self.state.select_prev_connection();
                        self.state.select_next_connection();
                    }
                    if matches!(e, KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }) {
                        break;
                    }
                }
                _ => ()
            }
            terminal.draw(|frame| self.draw(frame)).expect("failed to render");
        }
    }
}

