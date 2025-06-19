use crossterm::event::{self, KeyCode, KeyEvent};
use pages::main::MainPage;
use ratatui::{layout::Rect, prelude::Backend, Frame, Terminal};
use ui_mode::UserMode;

use crate::config::Config;

pub mod pages;
pub mod pipe;
pub mod ui_mode;

pub enum UiEvent {
    KeyboardEvent(KeyEvent),
}

pub enum WidgetReaction {
    ExitFromWidget,
    Nothing,
}

pub trait Widget<TerminalBackend>
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut Frame, rect: &Rect, user_mode: &UserMode, is_selected: bool);
    fn react_on_event(&mut self, terminal: &mut Terminal<TerminalBackend>, event: UiEvent, user_mode: &UserMode) -> WidgetReaction;
}

pub struct Renderer<TerminalBackend>
where
    TerminalBackend: Backend,
{
    pub widgets: Vec<(Rect, Box<dyn Widget<TerminalBackend>>, usize)>,
    pub selected_widget_index: Option<usize>,
    pub select_widget_index: usize,
    pub user_mode: UserMode,
    pub search_string: String,
}

impl<TerminalBackend> Renderer<TerminalBackend>
where
    TerminalBackend: Backend,
{
    pub fn new(widgets: Vec<(Rect, Box<dyn Widget<TerminalBackend>>, usize)>) -> Self {
        Self {
            widgets,
            selected_widget_index: None,
            select_widget_index: 0,
            user_mode: UserMode::Normal,
            search_string: "".to_string(),
        }
    }

    pub fn rerender(&mut self, frame: &mut Frame) {
        for (rect, widget, index) in &mut self.widgets {
            widget.render(frame, rect, &self.user_mode, *index == self.select_widget_index);
        }
    }

    pub fn run_event_loop(&mut self, terminal: &mut Terminal<TerminalBackend>) {
        if self.widgets.len() == 1 {
            self.selected_widget_index = Some(0)
        }
        loop {
            match event::read().expect("failed to read event") {
                event::Event::Key(key_event) => {
                    match &self.user_mode {
                        UserMode::Normal => {
                            match (key_event, &self.selected_widget_index) {
                                (KeyEvent { code: KeyCode::Char('h'), modifiers: _, kind: _, state: _ }, None) => {
                                    let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                    let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.x < select_widget.0.x).collect();
                                    match to_go_widgets.get(0) {
                                        Some(w) => {
                                            self.select_widget_index = w.2;
                                        },
                                        None => (),
                                    };
                                },
                                (KeyEvent { code: KeyCode::Char('j'), modifiers: _, kind: _, state: _ }, None) => {
                                    let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                    let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.y > select_widget.0.y).collect();
                                    match to_go_widgets.get(0) {
                                        Some(w) => {
                                            self.select_widget_index = w.2;
                                        },
                                        None => (),
                                    };
                                },
                                (KeyEvent { code: KeyCode::Char('k'), modifiers: _, kind: _, state: _ }, None) => {
                                    let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                    let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.y < select_widget.0.y).collect();
                                    match to_go_widgets.get(0) {
                                        Some(w) => {
                                            self.select_widget_index = w.2;
                                        },
                                        None => (),
                                    };
                                },
                                (KeyEvent { code: KeyCode::Char('l'), modifiers: _, kind: _, state: _ }, None) => {
                                    let select_widget = self.widgets.get(self.select_widget_index).unwrap();
                                    let to_go_widgets: Vec<_> = self.widgets.iter().filter(|w| w.0.x > select_widget.0.x).collect();
                                    match to_go_widgets.get(0) {
                                        Some(w) => {
                                            self.select_widget_index = w.2;
                                        },
                                        None => (),
                                    };
                                },
                                (KeyEvent { code: KeyCode::Char('i'), modifiers: _, kind: _, state: _ }, Some(_)) => {
                                    self.user_mode = UserMode::Insert;
                                },
                                (KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ }, None) => {
                                    self.selected_widget_index = Some(self.select_widget_index);
                                    continue;
                                },
                                (KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ }, Some(_)) => {
                                    self.selected_widget_index = None;
                                },
                                (KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ }, None) => {
                                    break;
                                },
                                _ => ()
                            };
                        },
                        UserMode::Insert => {
                            match key_event {
                                KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => self.user_mode = UserMode::Normal,
                                _ => ()
                            };
                        },
                        UserMode::SearchInput => {
                            match key_event {
                                KeyEvent { code: KeyCode::Char(ch), modifiers: _, kind: _, state: _ } => self.search_string.push(ch),
                                KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } => self.user_mode = UserMode::Normal,
                                KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => self.user_mode = UserMode::Search(self.search_string.clone(), 0),
                                _ => ()
                            };
                        },
                        UserMode::Search(search, num) => {
                            match key_event {
                                KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => (),
                                KeyEvent { code: KeyCode::Char('n'), modifiers: _, kind: _, state: _ } => self.user_mode = UserMode::Search(search.clone(), num + 1),
                                KeyEvent { code: KeyCode::Char('N'), modifiers: _, kind: _, state: _ } => self.user_mode = UserMode::Search(search.clone(), if *num > 0usize { num - 1} else { *num }),
                                _ => ()
                            };
                        },
                    };

                    match self.selected_widget_index {
                        Some(selected_widget_index) => {
                            match self.widgets[selected_widget_index].1.react_on_event(terminal, UiEvent::KeyboardEvent(key_event), &self.user_mode) {
                                WidgetReaction::ExitFromWidget => self.selected_widget_index = None,
                                WidgetReaction::Nothing => (),
                            };
                        },
                        None => (),
                    };
                },
                _ => ()
            };
            terminal.draw(|frame| self.rerender(frame)).expect("failed to render");
        }
    }
}

pub fn draw(config: Config) {
    let mut terminal = ratatui::init();
    
    let mut main_page = MainPage::new(&mut terminal, config);
    main_page.render(&mut terminal);
    main_page.run_event_loop(&mut terminal);
    // let mut query_page = QueryPage::new(&mut terminal);
    // query_page.render(&mut terminal);
    // query_page.run_event_loop(&mut terminal);

    ratatui::restore();
}

