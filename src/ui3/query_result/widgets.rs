use tuirealm::{AttrValue, Attribute, Event, MockComponent, event::{Key, KeyEvent}};

use crate::ui3::{AppEvent, EditorType, Msg};

pub enum WidgetContext {
    SearchPattern(String),
}

pub enum QueryResultWidget {
    Table(tui_realm_stdlib::Table)
}

impl QueryResultWidget {
    pub fn react_on_event(&mut self, context: Vec<WidgetContext>, event: Event<AppEvent>) -> Option<Msg> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
            Event::Keyboard(KeyEvent { code: Key::Char('H') | Key::Left, .. }) => Some(Msg::ToDbObjectsWidget),
            _ => match self {
                QueryResultWidget::Table(table) => table_react(table, context, event),
            }
        }
    }
}

fn table_react(table: &mut tui_realm_stdlib::Table, context: Vec<WidgetContext>, event: Event<AppEvent>) -> Option<Msg> {
    match event {
        Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
        Event::Keyboard(KeyEvent { code: Key::Char('H') | Key::Left, .. }) => Some(Msg::ToDbObjectsWidget),
        Event::Keyboard(KeyEvent { code: Key::Char('q'), ..}) => Some(Msg::ActivateEditor(EditorType::Query)),
        Event::Keyboard(KeyEvent { code: Key::Char('j') | Key::Down, ..}) => {
            table.states.incr_list_index(true);
            Some(Msg::None)
        },
        Event::Keyboard(KeyEvent { code: Key::Char('k') | Key::Up, ..}) => {
            table.states.decr_list_index(true);
            Some(Msg::None)
        },
        Event::Keyboard(KeyEvent { code: Key::Char('/'), .. }) => Some(Msg::ActivateEditor(EditorType::Search)),
        Event::Keyboard(KeyEvent { code: Key::Char('g'), .. }) => {
            table.states.list_index_at_first();
            Some(Msg::None)
        },
        Event::Keyboard(KeyEvent { code: Key::Char('G'), .. }) => {
            table.states.list_index_at_last();
            Some(Msg::None)
        },
        Event::Keyboard(KeyEvent { code: Key::Char('n'), ..}) => {
            for context in context {
                match context {
                    WidgetContext::SearchPattern(pattern) => {
                        let current_table = {
                            table.query(Attribute::Content).and_then(|val| {
                                match val {
                                    AttrValue::Table(table) => {
                                        let result: Vec<Vec<String>> = table.iter().map(|row| row.iter().map(|elem| elem.content.clone()).collect()).collect();
                                        Some(result)
                                    },
                                    _ => None
                                }
                            }).unwrap_or(vec![])
                        };

                        let start_index = table.states.list_index;
                        table.states.incr_list_index(true);
                        'searcher: while table.states.list_index != start_index {
                            let row = current_table.get(table.states.list_index).unwrap();
                            for item in row {
                                if item.contains(&pattern) {
                                    break 'searcher;
                                }
                            }
                            table.states.incr_list_index(true);
                        }
                    },
                }
            }
            Some(Msg::None)
        },
        Event::Keyboard(KeyEvent { code: Key::Char('N'), ..}) => {
            for context in context {
                match context {
                    WidgetContext::SearchPattern(pattern) => {
                        let current_table = {
                            table.query(Attribute::Content).and_then(|val| {
                                match val {
                                    AttrValue::Table(table) => {
                                        let result: Vec<Vec<String>> = table.iter().map(|row| row.iter().map(|elem| elem.content.clone()).collect()).collect();
                                        Some(result)
                                    },
                                    _ => None
                                }
                            }).unwrap_or(vec![])
                        };
                        let start_index = table.states.list_index;
                        table.states.decr_list_index(true);
                        'searcher: while table.states.list_index != start_index {
                            let row = current_table.get(table.states.list_index).unwrap();
                            for item in row {
                                if item.contains(&pattern) {
                                    break 'searcher;
                                }
                            }
                            table.states.decr_list_index(true);
                        }
                    },
                }
            }
            Some(Msg::None)
        }
        _ => Some(Msg::None)
    }
}

impl MockComponent for QueryResultWidget {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        match self {
            QueryResultWidget::Table(table) => table.view(frame, area),
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        match self {
            QueryResultWidget::Table(table) => table.query(attr),
        }
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match self {
            QueryResultWidget::Table(table) => table.attr(attr, value),
        }
    }

    fn state(&self) -> tuirealm::State {
        match self {
            QueryResultWidget::Table(table) => table.state(),
        }
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        match self {
            QueryResultWidget::Table(table) => table.perform(cmd),
        }
    }
}
