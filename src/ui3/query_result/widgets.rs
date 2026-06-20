use tuirealm::{
    event::{Key, KeyEvent},
    AttrValue, Attribute, Event, MockComponent,
};

use crate::ui3::{AppEvent, Id, Msg};
pub mod editor_popup;
mod editor_simple_input;
mod table;

pub enum WidgetContext {
    SearchPattern(String),
    Caller(Id),
}

pub enum QueryResultWidget {
    Table(tui_realm_stdlib::Table),
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiSelectorFor {
    Table(usize),
}

impl QueryResultWidget {
    pub fn react_on_event(
        &mut self,
        context: Vec<WidgetContext>,
        event: Event<AppEvent>,
    ) -> Option<Msg> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::ToConnectionsPage),
            Event::Keyboard(KeyEvent {
                code: Key::Char('H') | Key::Left,
                ..
            }) => Some(Msg::ToDbObjectsWidget),
            _ => match self {
                QueryResultWidget::Table(tbl) => table::table_react(tbl, context, event),
            },
        }
    }
}

impl MockComponent for QueryResultWidget {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        match self {
            QueryResultWidget::Table(table) => table.view(frame, area),
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        match self {
            QueryResultWidget::Table(table) => table.query(attr),
        }
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match self {
            QueryResultWidget::Table(table) => table.attr(attr, value),
        }
    }

    fn state(&self) -> tuirealm::State {
        match self {
            QueryResultWidget::Table(table) => table.state(),
        }
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        match self {
            QueryResultWidget::Table(table) => table.perform(cmd),
        }
    }
}
