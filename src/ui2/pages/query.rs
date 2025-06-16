use std::sync::{Arc, Mutex};

use ratatui::{layout::{Constraint, Layout, Rect}, prelude::Backend, Terminal};

use crate::{dbclient::fetcher::Fetcher, ui2::{pipe::Pipe, Renderer, Widget}};

use super::widgets::{db_objects::DbObjectsWidget, query_line::QueryLineWidget, query_result::QueryResultWidget};

pub struct QueryPage<Client, TerminalBackend>
where
    Client: Fetcher,
    TerminalBackend: Backend,
{
    pub renderer: Renderer<TerminalBackend>,
    pub fetcher: Arc<Mutex<Client>>,
}

impl<Client, TerminalBackend> QueryPage<Client, TerminalBackend>
where
    Client: Fetcher + 'static,
    TerminalBackend: Backend,
{
    pub fn new(terminal: &mut Terminal<impl Backend>, fetcher: Client) -> Self {
        let mut renderer: Option<Renderer<TerminalBackend>> = None;
        let fetcher_arc = Arc::new(Mutex::new(fetcher));
        terminal.draw(|frame| {

            let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]);
            let [units_list, query_area] = horizontal.areas(frame.area());
            let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Fill(10)]);
            let [query_line_area, query_result_area] = vertical.areas(query_area);

            let pipe = Arc::new(Mutex::new(Pipe::new()));
            let db_objects_widget = DbObjectsWidget::new(fetcher_arc.clone(), pipe.clone());
            let query_line_widget = QueryLineWidget::new(pipe.clone());
            let query_result_widget = QueryResultWidget::new(pipe.clone());

            let widgets: Vec<(Rect, Box<dyn Widget<TerminalBackend>>, usize)> = vec![
                (units_list, Box::new(db_objects_widget), 0),
                (query_line_area, Box::new(query_line_widget), 1),
                (query_result_area, Box::new(query_result_widget), 2),
            ];
            renderer = Some(Renderer::new(widgets))

        }).expect("render error");
        Self { renderer: renderer.unwrap(), fetcher: fetcher_arc }
    }

    pub fn render(&mut self, terminal: &mut Terminal<impl Backend>) {
        terminal.draw(|frame| self.renderer.rerender(frame)).expect("failed to render");
    }

    pub fn run_event_loop(&mut self, terminal: &mut Terminal<TerminalBackend>) {
        self.renderer.run_event_loop(terminal);
    }
}

