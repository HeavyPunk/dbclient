use std::fmt::Display;

use chrono::{DateTime, Utc};
use dbclient::Field;
use ratatui::{layout::Alignment, style::Color};
use tui_realm_stdlib::Input;
use tuirealm::{
    command::Cmd,
    event::{Key, KeyEvent},
    props::BorderType,
    AttrValue, Attribute, Component, Event, MockComponent,
};

use crate::{dbclient::fetcher::FetcherError, ui3::{AppEvent, Msg, query_result::widgets::editor_popup::EditorPopupWidget}};

pub struct EditorSimpleInput {
    component: Input,
    field: Field,
    pub editor_type: String,
}

impl EditorSimpleInput {
    pub fn new(title: impl Into<String> + Display, editor_type: impl Into<String>, field: Field) -> Self {
        Self {
            component: Input::default().title(format!("{}({})", title, field.as_type_str()), Alignment::Left).borders(
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
    fn get_content(&self) -> Result<Field, FetcherError> {
        let val = self.component.states.get_value();
        match &self.field {
            Field::String(s) => {
                if s.is_some() {
                    Ok(Field::String(Some(val)))
                } else if val != "" {
                    Ok(Field::String(None))
                } else {
                    Ok(Field::String(Some(val)))
                }
            },
            Field::StringContainer(items) => {
                if items.is_some() {
                    Ok(Field::StringContainer(Some(vec![val])))
                } else if val != "" {
                    Ok(Field::StringContainer(None))
                } else {
                    Ok(Field::StringContainer(Some(vec![val])))
                }
            },
            Field::Int8(i) => {
                if i.is_some() {
                    let val: i8 = val.parse()?;
                    Ok(Field::Int8(Some(val)))
                } else if val != "" {
                    Ok(Field::Int8(None))
                } else {
                    let val: i8 = val.parse()?;
                    Ok(Field::Int8(Some(val)))
                }
            },
            Field::Int16(i) => {
                if i.is_some() {
                    let val: i16 = val.parse()?;
                    Ok(Field::Int16(Some(val)))
                } else if val != "" {
                    Ok(Field::Int16(None))
                } else {
                    let val: i16 = val.parse()?;
                    Ok(Field::Int16(Some(val)))
                }
            },
            Field::Int32(i) => {
                if i.is_some() {
                    let val: i32 = val.parse()?;
                    Ok(Field::Int32(Some(val)))
                } else if val != "" {
                    Ok(Field::Int32(None))
                } else {
                    let val: i32 = val.parse()?;
                    Ok(Field::Int32(Some(val)))
                }
            },
            Field::Int64(i) => {
                if i.is_some() {
                    let val: i64 = val.parse()?;
                    Ok(Field::Int64(Some(val)))
                } else if val != "" {
                    Ok(Field::Int64(None))
                } else {
                    let val: i64 = val.parse()?;
                    Ok(Field::Int64(Some(val)))
                }
            },
            Field::Bool(b) => {
                if b.is_some() {
                    let val: bool = val.parse()?;
                    Ok(Field::Bool(Some(val)))
                } else if val != "" {
                    Ok(Field::Bool(None))
                } else {
                    let val: bool = val.parse()?;
                    Ok(Field::Bool(Some(val)))
                }
            },
            Field::Time(date_time) => {
                if date_time.is_some() {
                    let val: DateTime<Utc> = val.parse()?;
                    Ok(Field::Time(Some(val)))
                } else if val != "" {
                    Ok(Field::Time(None))
                } else {
                    let val: DateTime<Utc> = val.parse()?;
                    Ok(Field::Time(Some(val)))
                }
            },
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
