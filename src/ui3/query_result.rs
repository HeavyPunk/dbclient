use ratatui::{layout::Alignment, style::Color};
use tuirealm::{event::{Key, KeyEvent}, props::{BorderType, Borders, Table, TableBuilder, TextSpan}, Component, Event, MockComponent};

use crate::dbclient::fetcher::FetchResult;

use super::{AppEvent, Msg};

#[derive(MockComponent)]
pub struct QueryResult {
    component: tui_realm_stdlib::Table,
}

impl Default for QueryResult {
    fn default() -> Self {
        let table = tui_realm_stdlib::Table::default()
            .title("Result", Alignment::Left)
            .highlighted_color(Color::Yellow)
            .highlighted_str("> ")
            .scroll(true)
            .rewind(true)
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Yellow)
            );

        Self {
            component: table
        }
    }
}

impl Component<Msg, AppEvent> for QueryResult {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
            Event::Keyboard(KeyEvent { code: Key::Char('H'), ..}) => Some(Msg::ToDbObjectsWidget),
            Event::Keyboard(KeyEvent { code: Key::Char('K'), ..}) => Some(Msg::ToQueryInputWidget),
            Event::Keyboard(KeyEvent { code: Key::Char('j'), ..}) => {
                self.component.states.incr_list_index(true);
                Some(Msg::None)
            },
            Event::Keyboard(KeyEvent { code: Key::Char('k'), ..}) => {
                self.component.states.decr_list_index(true);
                Some(Msg::None)
            },
            _ => Some(Msg::None)
        }
    }
}

impl QueryResult {
    pub fn build_result_table(result: FetchResult) -> Table {
        if result.table.is_none() {
            return vec![];
        }
        let mut table_builder = TableBuilder::default();
        let table = result.table.unwrap();

        let headers: Vec<TextSpan> = table.1.keys().cloned().map(|key| TextSpan::new(key)).collect();
        for header in headers {
            table_builder.add_col(header);
        }
        // table_builder.add_row();

        let max_len = table.1.values().map(|v| v.len()).max().unwrap_or(0);
        for row_index in 0..max_len {
            table_builder.add_row();
            for column in table.1.values() {
                // let style = if self.selected_cell_index.0 == column_index && self.selected_cell_index.1 == row_index {
                //     Style::default().bg(Color::Yellow).fg(Color::Black)
                // } else {
                //     Style::default()
                // };
                let val = column.get(row_index).cloned().unwrap_or_else(|| "".to_string());
                table_builder.add_col(TextSpan::new(val));
            }
        }
        table_builder.build()
    }
}

