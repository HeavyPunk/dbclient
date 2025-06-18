use std::usize;

use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use query_result_widget::{QueryResultWidget, QueryResultWidgetState};
use query_widget::{QueryWidget, QueryWidgetState};
use ratatui::layout::Layout;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::widgets::{Block, Borders, List, Paragraph};

use crate::config::Connection;
use crate::dbclient::fetcher::{FetchRequest, Fetcher};
use crate::dbclient::query_builder::QueryElement;
use crate::ui::{command, editor};

use super::{InputMode, Screen, Widget};

pub mod query_widget;
pub mod query_result_widget;

#[derive(PartialEq)]
pub enum QueryWidgetSelector {
    DbObjects,
    QueryWidget,
    QueryResultWidget,
}

pub struct QueryState<'a, Fetch>
where
    Fetch: Fetcher,
{
    pub connection: &'a Connection,
    pub client: Fetch,
    pub query_string: String,
    pub query_widget: QueryWidgetSelector,
}


pub struct QueryPage<'a, Fetch>
where
    Fetch: Fetcher,
{
    state: &'a mut QueryState<'a, Fetch>,
    query_widget_state: QueryWidgetState,
    query_result_widget_state: QueryResultWidgetState,
}

impl<'a, Fetch> QueryPage<'a, Fetch>
where
    Fetch: Fetcher,
{
    pub fn new(state: &'a mut QueryState<'a, Fetch>) -> QueryPage<'a, Fetch> {
        QueryPage {
            state,
            query_widget_state: QueryWidgetState {
                input_mode: InputMode::Normal,
                query_string: String::from(""),
                command_string: String::from(""),
            },
            query_result_widget_state: QueryResultWidgetState {  },
        }
    }
}

impl<'a, Fetch> Screen for QueryPage<'a, Fetch>
where
    Fetch: Fetcher,
{
    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let horizontal = Layout::horizontal([Fill(1), Fill(3)]);
        let [units_list, query_area] = horizontal.areas(frame.area());
        let vertical = Layout::vertical([Fill(1), Fill(10)]);
        let [query_line_area, query_result_area] = vertical.areas(query_area);

        // TODO: to widget
        let client = &mut self.state.client;
        let db_objects = list_database_objects(client);
        let list = List::new(db_objects);
        frame.render_widget(list, units_list);

        let mut query_widget = QueryWidget::new(&mut self.query_widget_state, &query_line_area);
        query_widget.draw(frame, self.state.query_widget == QueryWidgetSelector::QueryWidget);

        let mut query_result_widget = QueryResultWidget::new(&mut self.query_result_widget_state, &query_result_area);
        query_result_widget.draw(frame, self.state.query_widget == QueryWidgetSelector::QueryResultWidget);

        // if self.state.input_mode == InputMode::Command {
        //     let center_layout = Layout::default()
        //         .direction(ratatui::layout::Direction::Vertical)
        //         .constraints([Fill(1), Length(3), Fill(1)].as_ref())
        //         .split(frame.area());
        //     let center_block_area = center_layout[1];
        //     let center_block = Block::new()
        //         .title("Centered Block")
        //         .borders(Borders::all());
        //     frame.render_widget(center_block, center_block_area);
        // }
    }

    fn handle_events(&mut self, terminal: &mut ratatui::Terminal<impl ratatui::prelude::Backend>) {
        loop {
            match event::read().expect("failed to read events") {
                event::Event::Key(key_event) => {
                    match key_event {
                        KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => break,
                        KeyEvent { code: KeyCode::Char('i'), modifiers: _, kind: _, state: _ } => {
                        },
                        KeyEvent { code: KeyCode::Char('w'), modifiers: KeyModifiers::CONTROL, kind: _, state: _ } => {
                            match event::read().expect("failed to read events") {
                                event::Event::Key(key_event) => match key_event {
                                    KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ } => {
                                        self.state.query_widget = QueryWidgetSelector::QueryWidget;
                                    },
                                    KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ } => {
                                        self.state.query_widget = QueryWidgetSelector::QueryResultWidget;
                                    },
                                    KeyEvent { code: KeyCode::Char('h'), modifiers: _, kind: _, state: _ } => {
                                        self.state.query_widget = QueryWidgetSelector::DbObjects;
                                    },
                                    KeyEvent { code: KeyCode::Char('l'), modifiers: _, kind: _, state: _ } => {
                                        self.state.query_widget = QueryWidgetSelector::QueryWidget;
                                    },
                                    _ => ()

                                },
                                _ => ()
                            }
                        }
                        KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => {
                            match self.state.query_widget {
                                QueryWidgetSelector::DbObjects => {
                                    unimplemented!()
                                },
                                QueryWidgetSelector::QueryWidget => {                                   
                                    // let mut query_widget = QueryWidget::new(&mut self.query_widget_state, &query_line_area);
                                },
                                QueryWidgetSelector::QueryResultWidget => todo!(),
                            }
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
            // if self.state.query_widget_selected {
            //
            // } else {
            //
            // }
            terminal.draw(|frame| self.draw(frame)).expect("failed to render");
        }
    }
}

fn list_database_objects(client: &mut impl Fetcher) -> Vec<String> {
    // let query = vec![QueryElement::Operator(String::from("KEYS")), QueryElement::Operator(String::from("*"))];
    let query = vec![QueryElement::RawQuery("KEYS *".to_string())];
    let fetch_result = client.fetch(&FetchRequest { query, limit: usize::MAX }).expect("client error");
    
    match fetch_result.table {
        Some(rows) => match rows.iter().last() {
            Some((_, vals)) => vals.to_vec(),
            None => vec![],
        },
        None => vec![],
    }
}
