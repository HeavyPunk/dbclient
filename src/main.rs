use crossterm::event::{self, Event};
use ratatui::{layout::Layout, text::Text, widgets::Block, Frame};


mod dbclient;

fn main() {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(draw_startup_menu).expect("failed to draw frame");
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break;
        }
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    frame.render_widget(text, frame.area());
}

fn draw_startup_menu(frame: &mut Frame) {
    use ratatui::layout::Constraint::{Fill, Length, Min};
    let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
    let [title_area, main_area, status_area] = vertical.areas(frame.area());
    let horizontal = Layout::horizontal([Fill(1); 2]);
    let [left_area, right_area] = horizontal.areas(main_area);
    frame.render_widget(Block::bordered().title("Select a connection"), title_area);
    frame.render_widget(Block::bordered().title("Status Bar"), status_area);
    frame.render_widget(Block::bordered().title("Left"), left_area);
    frame.render_widget(Block::bordered().title("Right"), right_area);
}

