use crossterm::event::{self, KeyCode, KeyEvent};
use pages::query::QueryPage;
use ratatui::{layout::Rect, prelude::Backend, Frame, Terminal};

use crate::config::{self, Config};

pub mod pages;

pub enum UiEvent {
    None,
    KeyboardEvent(KeyEvent),
}

pub enum WidgetReaction {
    ExitFromWidget,
    Nothing,
}

pub trait Widget {
    fn render(&mut self, frame: &mut Frame, rect: &Rect, is_selected: bool);
    fn react_on_event(&mut self, event: UiEvent) -> WidgetReaction;
}

pub struct Renderer {
    pub widgets: Vec<(Rect, Box<dyn Widget>, usize)>,
    pub selected_widget_index: Option<usize>,
    pub select_widget_index: usize,
}

impl Renderer {
    pub fn new(widgets: Vec<(Rect, Box<dyn Widget>, usize)>) -> Self {
        Self {
            widgets,
            selected_widget_index: None,
            select_widget_index: 0,
        }
    }

    pub fn rerender(&mut self, frame: &mut Frame) {
        // if let Some(selected_widget_index) = self.selected_widget_index {
        //     if let Some((rect, widget, _)) = self.widgets.get_mut(selected_widget_index) {
        //         widget.render(frame, rect, true);
        //     }
        //     return;
        // }

        for (rect, widget, index) in &mut self.widgets {
            widget.render(frame, rect, *index == self.select_widget_index);
        }
    }

    pub fn run_event_loop(&mut self, terminal: &mut Terminal<impl Backend>) {
        loop {
            match event::read().expect("failed to read event") {
                event::Event::Key(key_event) => {
                    if let Some(selected_widget_index) = self.selected_widget_index {
                        match self.widgets[selected_widget_index].1.react_on_event(UiEvent::KeyboardEvent(key_event)) {
                            WidgetReaction::ExitFromWidget => self.selected_widget_index = None,
                            WidgetReaction::Nothing => (),
                        };
                    } else {
                        match key_event {
                            KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => {
                                self.selected_widget_index = Some(self.select_widget_index);
                            },
                            KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => break,
                            KeyEvent { code: KeyCode::Char('h'), modifiers: _, kind: _, state: _ } => {
                                let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.x < select_widget.0.x).collect();
                                match to_go_widgets.get(0) {
                                    Some(w) => {
                                        self.select_widget_index = w.2;
                                    },
                                    None => (),
                                };
                            },
                            KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ } => {
                                let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.y > select_widget.0.y).collect();
                                match to_go_widgets.get(0) {
                                    Some(w) => {
                                        self.select_widget_index = w.2;
                                    },
                                    None => (),
                                };
                            },
                            KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ } => {
                                let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.y < select_widget.0.y).collect();
                                match to_go_widgets.get(0) {
                                    Some(w) => {
                                        self.select_widget_index = w.2;
                                    },
                                    None => (),
                                };
                            },
                            KeyEvent { code: KeyCode::Char('l'), modifiers: _, kind: _, state: _ } => {
                                let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.x > select_widget.0.x).collect();
                                match to_go_widgets.get(0) {
                                    Some(w) => {
                                        self.select_widget_index = w.2;
                                    },
                                    None => (),
                                };
                            },
                            _ => (),
                        };
                    }
                },
                _ => ()
            };
            terminal.draw(|frame| self.rerender(frame)).expect("failed to render");
        }
    }
}

pub fn draw(config: &Config) {
    let mut terminal = ratatui::init();
    
    let mut query_page = QueryPage::new(&mut terminal);
    query_page.render(&mut terminal);
    query_page.run_event_loop(&mut terminal);

    ratatui::restore();
}

