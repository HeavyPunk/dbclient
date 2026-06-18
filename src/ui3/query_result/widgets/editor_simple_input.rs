use dbclient::Field;
use ratatui::{layout::Alignment, style::Color};
use tui_realm_stdlib::Input;
use tuirealm::{
    command::Cmd,
    event::{Key, KeyEvent},
    props::BorderType,
    AttrValue, Attribute, Component, Event, MockComponent,
};

use crate::ui3::{query_result::widgets::editor_popup::EditorPopupWidget, AppEvent, Msg};

pub struct EditorSimpleInput {
    component: Input,
    field: Field,
    pub editor_type: String,
}

impl EditorSimpleInput {
    pub fn new(title: impl Into<String>, editor_type: impl Into<String>, field: Field) -> Self {
        Self {
            component: Input::default().title(title, Alignment::Left).borders(
                tuirealm::props::Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(Color::Yellow),
            ),
            editor_type: editor_type.into(),
            field: field,
        }
    }
}

impl Component<Msg, AppEvent> for EditorSimpleInput {
    fn on(&mut self, ev: tuirealm::Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code, .. }) => match code {
                Key::Esc => Some(Msg::DiactivateEditor),
                Key::Tab => Some(Msg::EditorPopupNext),
                Key::Enter => Some(Msg::EditorAccept),
                Key::Backspace => {
                    self.component.perform(Cmd::Delete);
                    Some(Msg::None)
                }
                Key::Char(c) => {
                    self.component.perform(Cmd::Type(c));
                    Some(Msg::None)
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl EditorPopupWidget for EditorSimpleInput {
    fn get_content(&self) -> Field {
        let val = self.component.states.get_value();
        if val != "" {
            Field::String(Some(self.component.states.get_value()))
        } else {
            Field::String(None)
        }
    }

    fn get_editor_type(&self) -> String {
        self.editor_type.clone()
    }
}

impl MockComponent for EditorSimpleInput {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        self.component.view(frame, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if let (Attribute::Focus, AttrValue::Flag(focused)) = (&attr, &value) {
            let border_color = if *focused { Color::Yellow } else { Color::Gray };
            self.component.attr(
                Attribute::Borders,
                AttrValue::Borders(tuirealm::props::Borders::default().color(border_color)),
            );
        }
        self.component.attr(attr, value);
    }

    fn state(&self) -> tuirealm::State {
        self.component.state()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        self.component.perform(cmd)
    }
}
