use std::{cmp::min, collections::HashMap, default, ops::Deref, path::Path, time::Duration, usize};
use ratatui::layout::{Alignment, Constraint, Direction, Rect};
use tuirealm::{props::{Layout, PropPayload, PropValue}, terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge}, Application, AttrValue, Attribute, EventListenerCfg, PollStrategy, Update};
use crate::{config::{Config, Connection}, dbclient::{dummy::DummyFetcher, fetcher::{FetchRequest, FetchResult, Fetcher}, query_builder::QueryElement, redis::{RedisConfig, RedisFetcher}}, ui3::{connections_list::ConnectionsListComponent, db_objects::DbObjects, editor_popup::EditorPopup, query_input::EditorInput, query_result::QueryResult, INPUT_POPUP_WIDGET_KIND}};

use super::{AppEvent, Id, Msg, Page, APP_SEARCH_PATTERN};


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

    pub fetcher: Option<Box<dyn Fetcher>>,
    pub query_page_selected_widget: Id,
    pub show_editor: bool,
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
                .tick_interval(Duration::from_millis(300))
        );

        assert!(app.mount(Id::ConnectionsList, Box::<ConnectionsListComponent>::default(), vec![]).is_ok());
        assert!(app.mount(Id::DbObjects, Box::<DbObjects>::default(), vec![]).is_ok());
        assert!(app.mount(Id::QueryResult, Box::<QueryResult>::default(), vec![]).is_ok());

        assert!(app.active(&Id::ConnectionsList).is_ok());

        Self {
            app,
            quit,
            redraw,
            terminal,
            connections: config.connections.clone(),
            selected_page: Page::Connections,
            fetcher: None,
            query_page_selected_widget: Id::DbObjects,
            show_editor: false,
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
                        self.app.view(&Id::QueryResult, f, chunks[1]);
                        if self.show_editor {
                            self.app.view(&Id::QueryLine, f, Self::centered_rect(80, 20, f.area()));
                        }
                    }).is_ok()
                );
            },
        };
    }

    fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
        Rect {
            x: area.width.saturating_sub(width) / 2,
            y: area.height.saturating_sub(height) / 2,
            width: min(width, area.width),
            height: min(height, area.height),
        }
    } 

    pub fn main_loop(&mut self) {
        while !self.quit {
            match self.app.tick(PollStrategy::Once) {
                Ok(messages) => {
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
        Some(Msg::None)
    }

    fn reload_query_result(&mut self, request: &FetchRequest) -> Option<Msg> {
        if let Some(ref mut fetcher) = self.fetcher {
            let result = fetcher.fetch(request).unwrap();
            assert!(
                self.app.attr(
                    &Id::QueryResult,
                    Attribute::Content,
                    AttrValue::Table(QueryResult::build_result_table(result))).is_ok()
            );
        }
        Some(Msg::FetchDbObjects)
    }

    fn fetch_db_object(&mut self, object: String) -> Option<Msg> {
        let query = FetchRequest {
            query: vec![QueryElement::ListAllItemsFrom(object)],
            limit: usize::MAX
        };
        return Some(Msg::ExecuteQuery(query));
    }

    fn add_db_object(&mut self, path: String, object_type: String, name: String) -> Option<Msg> {
        let query = FetchRequest {
            query: vec![QueryElement::AddDatabaseObject(path, object_type, name)],
            limit: usize::MAX
        };
        return Some(Msg::ExecuteQuery(query));
    }

    fn execute_custom_query(&mut self, query: String) -> Option<Msg> {
        let query = FetchRequest {
            query: vec![QueryElement::RawQuery(query)],
            limit: usize::MAX
        };
        return Some(Msg::ExecuteQuery(query));
    }

    fn search_pattern(&mut self, pattern: String) -> Option<Msg> {
        match self.query_page_selected_widget {
            Id::ConnectionsList => {
                assert!(self.app.attr(&Id::ConnectionsList, Attribute::Custom(APP_SEARCH_PATTERN), AttrValue::String(pattern)).is_ok());
                Some(Msg::None)
            },
            Id::DbObjects => {
                assert!(self.app.attr(&Id::DbObjects, Attribute::Custom(APP_SEARCH_PATTERN), AttrValue::String(pattern)).is_ok());
                Some(Msg::None)
            },
            Id::QueryLine => {
                assert!(self.app.attr(&Id::QueryLine, Attribute::Custom(APP_SEARCH_PATTERN), AttrValue::String(pattern)).is_ok());
                Some(Msg::None)
            },
            Id::QueryResult => {
                assert!(self.app.attr(&Id::QueryResult, Attribute::Custom(APP_SEARCH_PATTERN), AttrValue::String(pattern)).is_ok());
                Some(Msg::None)
            },
        }
    }

    fn reload_db_objects(&mut self) -> Option<Msg> {
        if let Some(ref mut fetcher) = self.fetcher {
            let result = fetcher.fetch_db_objects().unwrap();
            let result = result.table.unwrap_or((vec![], HashMap::default()));
            let list = result.1.iter().last();
            assert!(
                self.app.attr(
                    &Id::DbObjects,
                    Attribute::Content,
                    AttrValue::Table(DbObjects::build_objects_list(list.unwrap_or((&"".to_string(), &vec![])).1))).is_ok()
            );
        }
        Some(Msg::None)
    }

    fn init_fetcher(&mut self, selected_connection: usize) -> Option<Msg> {

        let connection = self.connections.get(selected_connection).unwrap();
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
        self.redraw = true;
        if let Some(msg) = msg {
            self.redraw = true;
            match msg {
                Msg::AppClose => {
                    self.quit = true;
                    None
                },
                Msg::ToQueryPage(selected_connection) => {
                    let result = self.init_fetcher(selected_connection);
                    assert!(self.app.active(&self.query_page_selected_widget).is_ok());
                    result
                },
                Msg::ToConnectionsPage => {
                    self.selected_page = Page::Connections;
                    assert!(self.app.active(&Id::ConnectionsList).is_ok());
                    None
                },
                Msg::FetchDbObjects => self.reload_db_objects(),

                Msg::FetchDbObject(object) => self.fetch_db_object(object),

                Msg::AddDbObject(path, object_type, name) => self.add_db_object(path, object_type, name),

                Msg::ExecuteCustomQuery(query) => self.execute_custom_query(query),

                Msg::SearchPattern(pattern) => self.search_pattern(pattern),

                Msg::ExecuteQuery(query) => self.reload_query_result(&query),
                
                Msg::ToDbObjectsWidget => {
                    self.query_page_selected_widget = Id::DbObjects;
                    assert!(self.app.active(&Id::DbObjects).is_ok());
                    None
                },

                Msg::ToQueryResultWidget => {
                    self.query_page_selected_widget = Id::QueryResult;
                    assert!(self.app.active(&Id::QueryResult).is_ok());
                    None
                },
                Msg::ActivateEditor(widget_kind) => {
                    self.show_editor = true;
                    assert!(self.app.mount(Id::QueryLine, Box::new(EditorPopup::new(widget_kind)), vec![]).is_ok());
                    assert!(self.app.active(&Id::QueryLine).is_ok());
                    None
                },
                Msg::DiactivateEditor => {
                    self.show_editor = false;
                    assert!(self.app.active(&self.query_page_selected_widget).is_ok());
                    if self.app.mounted(&Id::QueryLine) {
                        assert!(self.app.umount(&Id::QueryLine).is_ok());
                    }
                    None
                },

                Msg::EditorResult(editor_type, editors) => {
                    match editor_type {
                        super::EditorType::Search => {
                            let pattern = editors.get("search").unwrap_or(&vec![]).join("\n");
                            Some(Msg::SearchPattern(pattern))
                        },
                        super::EditorType::Query => {
                            let query = editors.get("query").unwrap_or(&vec![]).join("\n");
                            Some(Msg::ExecuteCustomQuery(query))
                        },
                        super::EditorType::AddDbObject => {
                            let root = editors.get("root").unwrap_or(&vec![]).join("\n");
                            let obj_type = editors.get("type").unwrap_or(&vec![]).join("\n");
                            let name = editors.get("name").unwrap_or(&vec![]).join("\n");
                            Some(Msg::AddDbObject(root, obj_type, name))
                        },
                    }
                },

                Msg::EditorPopupNext => {
                    assert!(self.app.active(&Id::QueryLine).is_ok());
                    None
                }

                Msg::None | Msg::EditorAccept => None,
            }
        } else {
            None
        }
    }
}

