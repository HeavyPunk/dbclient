use ratatui::{layout::Alignment, style::Color};
use tuirealm::{
    props::{BorderType, Borders, Table, TableBuilder, TextSpan},
    AttrValue, Attribute, Component, MockComponent,
};

use crate::{
    dbclient::fetcher::FetchResult,
    ui3::query_result::widgets::{QueryResultWidget, WidgetContext},
};

use super::{AppEvent, Msg, APP_SEARCH_PATTERN};

pub mod widgets;

pub const ATTRIBUTE_CONTENT_TYPE: &str = "attribute-content-type";

#[repr(isize)]
#[derive(Debug, PartialEq)]
pub enum ContentType {
    Table = 0,
}

#[derive(MockComponent)]
pub struct QueryResult {
    content_type: ContentType,
    component: QueryResultWidget,
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
                    .color(Color::Yellow),
            );

        Self {
            content_type: ContentType::Table,
            component: QueryResultWidget::Table(table),
        }
    }
}

impl Component<Msg, AppEvent> for QueryResult {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        let content_type = match self.query(Attribute::Custom(ATTRIBUTE_CONTENT_TYPE)) {
            Some(val) => match val {
                AttrValue::Number(content_type) => match content_type {
                    0 => ContentType::Table,
                    _ => return Some(Msg::None),
                },
                _ => return Some(Msg::None),
            },
            //NOTE: Nothing to display here, so travel to db objects widgets
            None => return Some(Msg::ToDbObjectsWidget),
        };

        let content = self.query(Attribute::Content);
        let focus = self.query(Attribute::Focus);

        if self.content_type != content_type {
            self.component = match content_type {
                ContentType::Table => {
                    let mut table = tui_realm_stdlib::Table::default()
                        .title("Result", Alignment::Left)
                        .highlighted_color(Color::Yellow)
                        .highlighted_str("> ")
                        .scroll(true)
                        .rewind(true)
                        .borders(
                            Borders::default()
                                .modifiers(BorderType::Rounded)
                                .color(Color::Yellow),
                        );
                    if let Some(content) = content {
                        table.attr(Attribute::Content, content);
                    }
                    if let Some(focus) = focus {
                        table.attr(Attribute::Focus, focus);
                    }
                    QueryResultWidget::Table(table)
                }
            };
        }

        let context = {
            let mut res = vec![];
            match self.query(Attribute::Custom(APP_SEARCH_PATTERN)) {
                Some(val) => match val {
                    AttrValue::String(pattern) => res.push(WidgetContext::SearchPattern(pattern)),
                    _ => {}
                },
                None => (),
            };
            res.push(WidgetContext::Caller(super::Id::QueryResult));
            res
        };
        self.component.react_on_event(context, ev)
    }
}

impl QueryResult {
    pub fn build_result_table(result: &FetchResult) -> Table {
        if let FetchResult::Table(Some(table)) = result {
            let mut table_builder = TableBuilder::default();

            let headers: Vec<TextSpan> = table
                .1
                .keys()
                .cloned()
                .map(|key| TextSpan::new(key))
                .collect();
            for header in headers {
                table_builder.add_col(header);
            }

            let max_len = table.1.values().map(|v| v.len()).max().unwrap_or(0);
            for row_index in 0..max_len {
                table_builder.add_row();
                for column in table.1.values() {
                    let val = column
                        .get(row_index)
                        .cloned()
                        .unwrap();
                    table_builder.add_col(
                        TextSpan::new(val.as_ui_value())
                    );
                }
            }
            table_builder.build()
        } else {
            vec![]
        }
    }
}
