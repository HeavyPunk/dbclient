use ratatui::{layout::{Constraint, Layout, Rect}, prelude::Backend, Terminal};

use crate::ui2::{Renderer, Widget};

use super::widgets::{db_objects::DbObjectsWidget, query_line::QueryLineWidget, query_result::QueryResultWidget};

pub struct QueryPage {
    pub renderer: Renderer,
}

impl QueryPage {
    pub fn new(terminal: &mut Terminal<impl Backend>) -> Self {
        let mut renderer: Option<Renderer> = None;
        terminal.draw(|frame| {

            let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]);
            let [units_list, query_area] = horizontal.areas(frame.area());
            let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Fill(10)]);
            let [query_line_area, query_result_area] = vertical.areas(query_area);
            let db_objects_widget = DbObjectsWidget::new();
            let query_line_widget = QueryLineWidget {};
            let query_result_widget = QueryResultWidget {};
            let widgets: Vec<(Rect, Box<dyn Widget>, usize)> = vec![
                (units_list, Box::new(db_objects_widget), 0),
                (query_line_area, Box::new(query_line_widget), 1),
                (query_result_area, Box::new(query_result_widget), 2),
            ];
            renderer = Some(Renderer::new(widgets))

        }).expect("render error");

        Self { renderer: renderer.unwrap() }
    }

    pub fn render(&mut self, terminal: &mut Terminal<impl Backend>) {
        terminal.draw(|frame| self.renderer.rerender(frame)).expect("failed to render");
    }

    pub fn run_event_loop(&mut self, terminal: &mut Terminal<impl Backend>) {
        self.renderer.run_event_loop(terminal);
    }
}

