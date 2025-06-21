use std::sync::{Arc, Mutex};

use ratatui::{prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Padding, Paragraph}};

use crate::ui2::{pipe::Pipe, ui_mode::UserMode, Widget};

pub struct UserModeWidget {
    pipe: Arc<Mutex<Pipe>>,
}

impl UserModeWidget {
    pub fn new(pipe: Arc<Mutex<Pipe>>) -> Self {
        Self { pipe }
    }
}

impl<TerminalBackend> Widget<TerminalBackend> for UserModeWidget
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut ratatui::Frame, rect: &ratatui::prelude::Rect, user_mode: &UserMode, _: bool) {
        let (text, color) = match user_mode {
            UserMode::Normal => ("NORMAL".to_string(), Color::LightBlue),
            UserMode::Insert => ("INSERT".to_string(), Color::LightGreen),
            UserMode::SearchInput => ("SEARCH".to_string(), Color::Yellow),
            UserMode::Search(_, _) => ("SEARCH".to_string(), Color::Yellow),
            UserMode::Command => ({
                let mut pipe = self.pipe.lock().unwrap();
                match pipe.try_get_user_mode() {
                    Ok(cmd) => format!("COMMAND :{}", cmd),
                    Err(_) => "COMMAND".to_string(),
                }
            }, Color::Magenta),
        };
        let user_mode = Paragraph::new(text)
            .block(Block::new()
                .padding(Padding::left(1))
                .borders(Borders::NONE))
            .style(Style::default().bg(color).fg(Color::Black));
        frame.render_widget(user_mode, *rect);
    }

    fn react_on_event(&mut self, _: &mut ratatui::Terminal<TerminalBackend>, _: crate::ui2::UiEvent, _: &UserMode) -> crate::ui2::WidgetReaction {
        crate::ui2::WidgetReaction::ExitFromWidget
    }
}

