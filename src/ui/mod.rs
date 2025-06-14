use ratatui::{prelude::Backend, Terminal};
use screens::{main::{MainPage, MainState}, Screen};
use crate::config::Config;

mod screens;
mod editor;
mod command;

pub fn draw(config: &Config) {
    let mut terminal = ratatui::init();

    let mut state = MainState {
        connections_list: &config.connections,
        selected_connection_index: 0
    };

    let main_page = MainPage::new(&mut state);

    draw_screen(&mut terminal, main_page, config);

    ratatui::restore();
}

fn draw_screen(terminal: &mut Terminal<impl Backend>, mut screen: impl Screen, config: &Config) {
    terminal.draw(|frame| screen.draw(frame)).expect("failed to draw frame");
    screen.handle_events(terminal);
}

