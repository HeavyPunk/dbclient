use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, prelude::Backend, style::{Color, Style}, text::Text, widgets::{Block, Borders, List, Row}, Frame, Terminal};

use crate::{dbclient::{fetcher::{FetchRequest, Fetcher}, query_builder::QueryElement}, ui2::Widget};

pub struct DbObjectsWidget<Client>
where
    Client: Fetcher
{
    selected_object_index: usize,
    db_objects: Option<Vec<String>>,
    fetcher: Arc<Mutex<Client>>,
}

impl<'a, Client> DbObjectsWidget<Client>
where
    Client: Fetcher,
{
    pub fn new(fetcher: Arc<Mutex<Client>>) -> Self {
        Self {
            selected_object_index: 0,
            db_objects: None,
            fetcher
        }
    }
}

impl<Client, TerminalBackend> Widget<TerminalBackend> for DbObjectsWidget<Client>
where
    Client: Fetcher,
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut Frame, rect: &Rect, is_selected: bool) {
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        if let None = self.db_objects {
            self.db_objects = Some(list_database_objects());
        }

        let buf: Vec<Text<'_>> = self.db_objects.as_ref().unwrap()
            .iter()
            .enumerate()
            .map(|(index, elem)| {
                let style = if index == self.selected_object_index {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                Text::from(elem.clone()).style(style)
            })
            .collect();
        let list = List::new(buf)
            .block(Block::new().title("Database Objects").borders(Borders::all()).style(style));
        frame.render_widget(list, *rect);
    }

    fn react_on_event(&mut self, _: &mut Terminal<TerminalBackend>, event: crate::ui2::UiEvent) -> crate::ui2::WidgetReaction {
        match event {
            crate::ui2::UiEvent::None => crate::ui2::WidgetReaction::Nothing,
            crate::ui2::UiEvent::KeyboardEvent(key_event) => {
                match key_event {
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => crate::ui2::WidgetReaction::ExitFromWidget,
                    KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => crate::ui2::WidgetReaction::Nothing, //TODO: list object items
                    KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ } => {
                        if let Some(db_objects) = &self.db_objects {
                            if self.selected_object_index < db_objects.len() - 1 {
                                self.selected_object_index += 1;
                            }
                        }
                        crate::ui2::WidgetReaction::Nothing
                    },
                    KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ } => {
                        if self.selected_object_index > 0 {
                            self.selected_object_index -= 1;
                        }
                        crate::ui2::WidgetReaction::Nothing
                    },
                    _ => crate::ui2::WidgetReaction::Nothing,

                }
            },
        }
    }
}

// fn list_database_objects(client: &mut impl Fetcher) -> Vec<String> {
fn list_database_objects() -> Vec<String> {
    vec![String::from("users"), String::from("tokens"), String::from("files")]
    // let query = vec![QueryElement::Operator(String::from("KEYS")), QueryElement::Operator(String::from("*"))];
    // let fetch_result = client.fetch(&FetchRequest { query, limit: usize::MAX }).expect("client error");
    //
    // match fetch_result.rows {
    //     Some(rows) => rows.iter().map(|row| row.columns.join(" ").to_string()).collect(),
    //     None => vec![],
    // }
}


