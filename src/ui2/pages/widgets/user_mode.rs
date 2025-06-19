use ratatui::{prelude::Backend, style::{Color, Style}, widgets::{Block, Borders, Padding, Paragraph}};

use crate::ui2::Widget;

pub struct UserModeWidget {

}

impl UserModeWidget {
    pub fn new() -> Self {
        Self {  }
    }
}

impl<TerminalBackend> Widget<TerminalBackend> for UserModeWidget
where
    TerminalBackend: Backend,
{
    fn render(&mut self, frame: &mut ratatui::Frame, rect: &ratatui::prelude::Rect, user_mode: &crate::ui2::ui_mode::UserMode, _: bool) {
        let (text, color) = match user_mode {
            crate::ui2::ui_mode::UserMode::Normal => ("NORMAL", Color::LightBlue),
            crate::ui2::ui_mode::UserMode::Insert => ("INSERT", Color::LightGreen),
            crate::ui2::ui_mode::UserMode::SearchInput => ("SEARCH", Color::Yellow),
            crate::ui2::ui_mode::UserMode::Search(_, _) => ("SEARCH", Color::Yellow),
        };
        let user_mode = Paragraph::new(text)
            .block(Block::new()
                .padding(Padding::left(1))
                .borders(Borders::NONE))
            .style(Style::default().bg(color).fg(Color::Black));
        frame.render_widget(user_mode, *rect);
    }

    fn react_on_event(&mut self, _: &mut ratatui::Terminal<TerminalBackend>, _: crate::ui2::UiEvent, _: &crate::ui2::ui_mode::UserMode) -> crate::ui2::WidgetReaction {
        crate::ui2::WidgetReaction::ExitFromWidget
    }
}

