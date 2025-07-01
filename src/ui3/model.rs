use std::{sync::{Arc, Mutex}, time::Duration};

use ratatui::layout::{Constraint, Direction};
use tuirealm::{listener::Poll, props::{Layout, PropPayload, PropValue}, terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge}, Application, AttrValue, Attribute, EventListenerCfg, PollStrategy, Update};

use crate::{config::{Config, Connection}, dbclient::{dummy::DummyFetcher, fetcher::Fetcher, redis::{RedisConfig, RedisFetcher}}, ui3::{connections_list::ConnectionsListComponent, db_objects::DbObjects}};

use super::{AppEvent, Id, Msg, Page};


pub struct Model<TermAdapter>
where
    TermAdapter: TerminalAdapter,
{
    pub app: Application<Id, Msg, AppEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<TermAdapter>,
    pub selected_page: Page,

    pub connections: Vec<Connection>,
    pub selected_connection_index: usize,

    pub fetcher: Option<Box<dyn Fetcher>>,
    pub selected_db_object: usize,
    pub db_objects_count: usize,
}

impl Model<CrosstermTerminalAdapter> {
    pub fn new(config: &Config) -> Self {
        let quit = false;
        let redraw = true;
        let mut terminal = TerminalBridge::init_crossterm().expect("Cannot create terminal bridge");

        let _ = terminal.enable_raw_mode();
        let _ = terminal.enter_alternate_screen();

        let mut app = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(10), 3)
                .poll_timeout(Duration::from_millis(20))
                .tick_interval(Duration::from_secs(1))
        );

        assert!(
            app.mount(Id::ConnectionsList, Box::<ConnectionsListComponent>::default(), vec![]).is_ok()
        );
        assert!(app.mount(Id::DbObjects, Box::<DbObjects>::default(), vec![]).is_ok());
        assert!(app.active(&Id::ConnectionsList).is_ok());

        Self {
            app,
            quit,
            redraw,
            terminal,
            connections: config.connections.clone(),
            selected_connection_index: 0,
            selected_page: Page::Connections,
            fetcher: None,
            selected_db_object: 0,
            db_objects_count: 0,
        }
    }

    pub fn view(&mut self) {
        match self.selected_page {
            Page::Connections => {
                assert!(self
                    .terminal
                    .raw_mut()
                    .draw(|f| {
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(1)
                            .constraints(
                                [
                                    Constraint::Fill(1),
                                ].as_ref(),
                            ).chunks(f.area());
                        self.app.view(&Id::ConnectionsList, f, chunks[0]);
                    }).is_ok()
                );
                assert!(self.app.active(&Id::ConnectionsList).is_ok());
                self.reload_connections();
            },
            Page::Query => {
                assert!(self
                    .terminal
                    .raw_mut()
                    .draw(|f| {
                        let chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .margin(1)
                            .constraints(
                                [
                                    Constraint::Fill(1),
                                    Constraint::Fill(4),
                                ].as_ref(),
                            ).chunks(f.area());
                        self.app.view(&Id::DbObjects, f, chunks[0]);
                    }).is_ok()
                );
                assert!(self.app.active(&Id::DbObjects).is_ok());
            },
        };
    }

    pub fn main_loop(&mut self) {
        while !self.quit {
            match self.app.tick(PollStrategy::Once) {
                Ok(messages) => {
                    self.redraw = true;
                    messages.iter().map(Some).for_each(|msg| {
                        let mut msg = msg.cloned();
                        while msg.is_some() {
                            msg = self.update(msg);
                        }
                    });
                },
                Err(e) => panic!("{e}"),
            };

            if self.redraw {
                self.view();
                self.redraw = false;
            }
        }
        let _ = self.terminal.leave_alternate_screen();
        let _ = self.terminal.disable_raw_mode();
        let _ = self.terminal.clear_screen();
    }

    fn reload_connections(&mut self) -> Option<Msg> {
        assert!(
            self.app
                .attr(
                    &Id::ConnectionsList,
                    Attribute::Content,
                    AttrValue::Table(ConnectionsListComponent::build_connections_table(&self.connections))).is_ok()
        );
        assert!(
            self.app.attr(&Id::ConnectionsList, Attribute::Value, AttrValue::Payload(PropPayload::One(PropValue::Usize(self.selected_connection_index)))).is_ok()
        );
        Some(Msg::None)
    }

    fn reload_db_objects(&mut self) -> Option<Msg> {
        if let Some(ref mut fetcher) = self.fetcher {
            let result = fetcher.fetch_db_objects().unwrap();
            let result = result.table.unwrap();
            let list = result.1.iter().last().unwrap();
            self.db_objects_count = list.1.len();
            assert!(
                self.app.attr(
                    &Id::DbObjects,
                    Attribute::Content,
                    AttrValue::Table(DbObjects::build_objects_list(list.1))).is_ok()
            );
        }
        Some(Msg::None)
    }

    fn select_db_object(&mut self) -> Option<Msg> {
        assert!(self.app.attr(&Id::DbObjects, Attribute::Value, AttrValue::Payload(PropPayload::One(PropValue::Usize(self.selected_db_object)))).is_ok());
        Some((Msg::None))
    }

    fn init_fetcher(&mut self) -> Option<Msg> {

        let connection = self.connections.get(self.selected_connection_index).unwrap();
        let fetcher: Box<dyn Fetcher> = match connection.connection_type {
            crate::config::ConnectionType::Redis => {
                Box::new(RedisFetcher {
                    config: RedisConfig { uri: connection.connection_string.clone() }
                })
            },
            crate::config::ConnectionType::Postgres => Box::new(DummyFetcher::new()),
            crate::config::ConnectionType::MySql => Box::new(DummyFetcher::new()),
        };

        self.selected_page = Page::Query;
        self.fetcher = Some(fetcher);
        Some(Msg::FetchDbObjects)
    }
}

impl Update<Msg> for Model<CrosstermTerminalAdapter>
{
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            self.redraw = true;
            match msg {
                Msg::AppClose => {
                    self.quit = true;
                    None
                },
                Msg::ConnectionSelected(index) => {
                    self.selected_connection_index = index;
                    self.reload_connections()
                },
                Msg::SelectPrevConnection => {
                    if self.selected_connection_index > 0 {
                        self.selected_connection_index -= 1;
                        self.reload_connections()
                    } else {
                        None
                    }
                },
                Msg::SelectNextConnection => {
                    if self.selected_connection_index < self.connections.len() - 1 {
                        self.selected_connection_index += 1;
                        self.reload_connections()
                    } else {
                        None
                    }
                },
                Msg::ToQueryPage => {
                    self.init_fetcher()
                },
                Msg::ToConnectionsPage => {
                    self.selected_page = Page::Connections;
                    None
                },
                Msg::FetchDbObjects => {
                    self.reload_db_objects()
                },
                Msg::SelectNextDbObject => {
                    if self.selected_db_object < self.db_objects_count - 1 {
                        self.selected_db_object += 1;
                        self.select_db_object()
                    } else {
                        None
                    }
                },
                Msg::SelectPrevDbObject => {
                    if self.selected_db_object > 0 {
                        self.selected_db_object -= 1;
                        self.select_db_object()
                    } else {
                        None
                    }
                },

                Msg::None => None,
            }
        } else {
            None
        }
    }
}

