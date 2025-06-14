use ratatui::{layout::Rect, prelude::Backend, Frame, Terminal};

pub mod main;
pub mod query;

pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_events(&mut self, terminal: &mut Terminal<impl Backend>);
}

pub trait Widget {
    fn draw(&mut self, frame: &mut Frame, is_selected: bool);
    fn handle_events(&mut self, terminal: &mut Terminal<impl Backend>);
    fn get_content(&mut self) -> Content;
}

pub enum Content {
    SimpleString(String)
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Command,
}

