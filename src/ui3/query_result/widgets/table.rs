use tuirealm::{
    event::{Key, KeyEvent},
    AttrValue, Attribute, Event, MockComponent,
};

use crate::ui3::{query_result::widgets::WidgetContext, AppEvent, EditorType, Msg};

pub fn table_react(
    table: &mut tui_realm_stdlib::Table,
    context: Vec<WidgetContext>,
    event: Event<AppEvent>,
) -> Option<Msg> {
    match event {
        Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
        Event::Keyboard(KeyEvent {
            code: Key::Char('H') | Key::Left,
            ..
        }) => Some(Msg::ToDbObjectsWidget),
        Event::Keyboard(KeyEvent {
            code: Key::Char('q'),
            ..
        }) => {
            for context in context {
                match context {
                    WidgetContext::Caller(id) => {
                        return Some(Msg::ActivateEditor(
                            id,
                            EditorType::Query,
                            super::UiSelectorFor::Table(table.states.list_index - 1),
                        ))
                    }
                    _ => (),
                }
            }
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('j') | Key::Down,
            ..
        }) => {
            table.states.incr_list_index(true);
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('k') | Key::Up,
            ..
        }) => {
            table.states.decr_list_index(true);
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('/'),
            ..
        }) => {
            for context in context {
                match context {
                    WidgetContext::Caller(id) => {
                        return Some(Msg::ActivateEditor(
                            id,
                            EditorType::Search,
                            super::UiSelectorFor::Table(table.states.list_index),
                        ))
                    }
                    _ => (),
                }
            }
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            ..
        }) => {
            table.states.list_index_at_first();
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('G'),
            ..
        }) => {
            table.states.list_index_at_last();
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('n'),
            ..
        }) => {
            for context in context {
                match context {
                    WidgetContext::SearchPattern(pattern) => {
                        let current_table = {
                            table
                                .query(Attribute::Content)
                                .and_then(|val| match val {
                                    AttrValue::Table(table) => {
                                        let result: Vec<Vec<String>> = table
                                            .iter()
                                            .map(|row| {
                                                row.iter()
                                                    .map(|elem| elem.content.clone())
                                                    .collect()
                                            })
                                            .collect();
                                        Some(result)
                                    }
                                    _ => None,
                                })
                                .unwrap_or(vec![])
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
                    }
                    _ => (),
                }
            }
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('N'),
            ..
        }) => {
            for context in context {
                match context {
                    WidgetContext::SearchPattern(pattern) => {
                        let current_table = {
                            table
                                .query(Attribute::Content)
                                .and_then(|val| match val {
                                    AttrValue::Table(table) => {
                                        let result: Vec<Vec<String>> = table
                                            .iter()
                                            .map(|row| {
                                                row.iter()
                                                    .map(|elem| elem.content.clone())
                                                    .collect()
                                            })
                                            .collect();
                                        Some(result)
                                    }
                                    _ => None,
                                })
                                .unwrap_or(vec![])
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
                    }
                    _ => (),
                }
            }
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('O'),
            ..
        }) => {
            for context in context {
                match context {
                    WidgetContext::Caller(id) => {
                        return Some(Msg::ActivateEditor(
                            id,
                            EditorType::AddRecord,
                            super::UiSelectorFor::Table(table.states.list_index - 1),
                        ))
                    }
                    _ => (),
                }
            }
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Char('i'),
            ..
        }) => {
            for context in context {
                match context {
                    WidgetContext::Caller(id) => {
                        return Some(Msg::ActivateEditor(
                            id,
                            EditorType::UpdateRecord,
                            super::UiSelectorFor::Table(table.states.list_index - 1),
                        ))
                    }
                    _ => (),
                }
            }
            Some(Msg::None)
        }
        _ => Some(Msg::None),
    }
}
