use ratatui::{layout::Alignment, style::Color};
use tuirealm::{event::{Key, KeyEvent}, props::{BorderType, Borders, Table, TableBuilder, TextSpan}, AttrValue, Attribute, Component, Event, MockComponent};

use crate::dbclient::fetcher::FetchResult;

use super::{AppEvent, EditorType, Msg, APP_SEARCH_PATTERN};

#[derive(MockComponent)]
pub struct QueryResult {
    component: tui_realm_stdlib::Table,
}

impl Default for QueryResult {
    fn default() -> Self {
        let table = tui_realm_stdlib::Table::default()
            .title("Result", Alignment::Left)
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
            component: table
        }
    }
}

impl Component<Msg, AppEvent> for QueryResult {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
            Event::Keyboard(KeyEvent { code: Key::Char('H') | Key::Left, ..}) => Some(Msg::ToDbObjectsWidget),
            Event::Keyboard(KeyEvent { code: Key::Char('q'), ..}) => Some(Msg::ActivateEditor(EditorType::Query)),
            Event::Keyboard(KeyEvent { code: Key::Char('j') | Key::Down, ..}) => {
                self.component.states.incr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('k') | Key::Up, ..}) => {
                self.component.states.decr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('/'), .. }) => Some(Msg::ActivateEditor(EditorType::Search)),
            Event::Keyboard(KeyEvent { code: Key::Char('g'), .. }) => {
                self.component.states.list_index_at_first();
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('G'), .. }) => {
                self.component.states.list_index_at_last();
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('n'), ..}) => {
                self.query(Attribute::Custom(APP_SEARCH_PATTERN))
                    .and_then(|val| match val {
                        AttrValue::String(pattern) => {
                            let current_table = self.get_current_table();

                            let start_index = self.component.states.list_index;
                            self.component.states.incr_list_index(true);
                            'searcher: while self.component.states.list_index != start_index {
                                let row = current_table.get(self.component.states.list_index).unwrap();
                                for item in row {
                                    if item.contains(&pattern) {
                                        break 'searcher;
                                    }
                                }
                                self.component.states.incr_list_index(true);
                            }
                            None
                        },
                        _ => None
                    }).unwrap_or(Some(Msg::None))
            },
            Event::Keyboard(KeyEvent { code: Key::Char('N'), ..}) => {
                self.query(Attribute::Custom(APP_SEARCH_PATTERN))
                    .and_then(|val| match val {
                        AttrValue::String(pattern) => {
                            let current_table = self.get_current_table();

                            let start_index = self.component.states.list_index;
                            self.component.states.decr_list_index(true);
                            'searcher: while self.component.states.list_index != start_index {
                                let row = current_table.get(self.component.states.list_index).unwrap();
                                for item in row {
                                    if item.contains(&pattern) {
                                        break 'searcher;
                                    }
                                }
                                self.component.states.decr_list_index(true);
                            }
                            None
                        },
                        _ => None
                    }).unwrap_or(Some(Msg::None))
            }
            _ => Some(Msg::None)
        }
    }
}

impl QueryResult {
    pub fn build_result_table(result: FetchResult) -> Table {
        if result.table.is_none() {
            return vec![];
        }
        let mut table_builder = TableBuilder::default();
        let table = result.table.unwrap();

        let headers: Vec<TextSpan> = table.1.keys().cloned().map(|key| TextSpan::new(key)).collect();
        for header in headers {
            table_builder.add_col(header);
        }

        let max_len = table.1.values().map(|v| v.len()).max().unwrap_or(0);
        for row_index in 0..max_len {
            table_builder.add_row();
            for column in table.1.values() {
                let val = column.get(row_index).cloned().unwrap_or_else(|| "".to_string());
                table_builder.add_col(TextSpan::new(val));
            }
        }
        table_builder.build()
    }

    pub fn get_current_table(&self) -> Vec<Vec<String>> {
        self.component.query(Attribute::Content).and_then(|val| {
            match val {
                AttrValue::Table(table) => {
                    let result: Vec<Vec<String>> = table.iter().map(|row| row.iter().map(|elem| elem.content.clone()).collect()).collect();
                    Some(result)
                },
                _ => None
            }
        }).unwrap_or(vec![])
    }
}

