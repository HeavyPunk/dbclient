use ratatui::{layout::Rect, prelude::Backend, Terminal};

use crate::{config::Config, ui2::{Renderer, Widget}};

use super::widgets::connections_list::ConnectionsListWidget;


pub struct MainPage<TerminalBackend>
where
    TerminalBackend: Backend,
{
    renderer: Renderer<TerminalBackend>,
}

impl<TerminalBackend> MainPage<TerminalBackend>
where
    TerminalBackend: Backend,
{
    pub fn new(terminal: &mut Terminal<impl Backend>, config: Config) -> Self {
        let mut renderer: Option<Renderer<TerminalBackend>> = None;
        terminal.draw(|frame| {
            let connections_list = ConnectionsListWidget::new(config.connections);
            let widgets: Vec<(Rect, Box<dyn Widget<TerminalBackend>>, usize)> = vec![
                (frame.area(), Box::new(connections_list), 0)
            ];
            renderer = Some(Renderer::new(widgets))
        }).expect("render error");
        Self { renderer: renderer.unwrap() }
    }

    pub fn render(&mut self, terminal: &mut Terminal<impl Backend>) {
        terminal.draw(|frame| self.renderer.rerender(frame)).expect("failed to render");
    }

    pub fn run_event_loop(&mut self, terminal: &mut Terminal<TerminalBackend>) {
        self.renderer.run_event_loop(terminal);
    }
}


