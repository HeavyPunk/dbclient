use std::collections::HashMap;

use dbclient::Field;
use ratatui::{
    layout::{Constraint, Direction as RatatuiDirection, Layout},
    style::Color,
    widgets::{Block, Borders, Clear},
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::KeyEvent,
    props::BorderType,
    AttrValue, Attribute, Component, Event, MockComponent,
};

use crate::{
    dbclient::fetcher::{FetchResult, FetcherError},
    ui3::{
        self,
        query_result::widgets::{editor_simple_input::EditorSimpleInput, UiSelectorFor},
        Id,
    },
};

use super::{AppEvent, Msg};

pub trait EditorPopupWidget: Component<Msg, AppEvent> {
    fn get_content(&self) -> Result<Field, FetcherError>;
    fn get_editor_type(&self) -> String;
}

pub enum EditorType {
    Multiline,
    Oneline,
}

pub struct EditorPopup {
    components: Vec<(Box<dyn EditorPopupWidget>, EditorType)>,
    fetch_result_internal_selector: UiSelectorFor,
    selected_component_index: usize,
    editor_type: ui3::EditorType,
    caller: Id,
}

impl EditorPopup {
    pub fn new(
        fetch_result: &FetchResult,
        entity_selector: UiSelectorFor,
        editor_type: ui3::EditorType,
        caller: Id,
    ) -> Self {
        let components: Vec<(Box<dyn EditorPopupWidget>, EditorType)> =
            match (fetch_result, entity_selector.clone()) {
                (FetchResult::Table(Some(table)), UiSelectorFor::Table(row_index)) => {
                    let inputs: Vec<(Box<dyn EditorPopupWidget>, EditorType)> = table
                        .1
                        .iter()
                        .map(|pair| (pair.0, pair.1.get(row_index)))
                        .filter(|pair| pair.1.is_some())
                        .map(|pair| -> (Box<dyn EditorPopupWidget>, EditorType) {
                            let mut field = pair.1.unwrap().clone();
                            if editor_type == ui3::EditorType::AddRecord {
                                field = field.as_only_type();
                            }
                            (
                                Box::new(EditorSimpleInput::new(pair.0, pair.0, field)),
                                EditorType::Oneline,
                            )
                        })
                        .collect();
                    inputs
                }
                _ => vec![],
            };

        let mut popup = Self {
            components,
            fetch_result_internal_selector: entity_selector,
            selected_component_index: 0,
            editor_type,
            caller,
        };

        popup.update_focus();
        popup
    }

    fn update_focus(&mut self) {
        for (component, _) in &mut self.components {
            component.attr(Attribute::Focus, AttrValue::Flag(false));
        }

        if let Some((component, _)) = self.components.get_mut(self.selected_component_index) {
            component.attr(Attribute::Focus, AttrValue::Flag(true));
        }
    }

    fn next_component(&mut self) {
        self.selected_component_index = (self.selected_component_index + 1) % self.components.len();
        self.update_focus();
    }

    fn prev_component(&mut self) {
        if self.selected_component_index == 0 {
            self.selected_component_index = self.components.len() - 1;
        } else {
            self.selected_component_index -= 1;
        }
        self.update_focus();
    }

    // fn get_title(&self) -> &'static str {
    //     match self.editor_type {
    //         super::EditorType::Search => "Search",
    //         super::EditorType::Query => "Query Editor",
    //         super::EditorType::AddDbObject => "Add Database Object",
    //     }
    // }
}

impl Component<Msg, AppEvent> for EditorPopup {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        if let Event::Keyboard(KeyEvent {
            code, modifiers, ..
        }) = &ev
        {
            match code {
                tuirealm::event::Key::Tab => {
                    if modifiers.contains(tuirealm::event::KeyModifiers::SHIFT) {
                        self.prev_component();
                    } else {
                        self.next_component();
                    }
                    return Some(Msg::None);
                }
                _ => {}
            }
        }

        let (component, _) = self
            .components
            .get_mut(self.selected_component_index)
            .unwrap();
        match component.on(ev) {
            Some(Msg::EditorAccept) => {
                let editors_results: Result<HashMap<String, Field>, FetcherError> = self
                    .components
                    .iter()
                    .map(|c| {
                        let content = c.0.get_content();
                        match content {
                            Ok(f) => Ok((c.0.get_editor_type(), f)),
                            Err(e) => Err(e),
                        }
                    })
                    .collect();
                match editors_results {
                    Ok(r) => Some(Msg::EditorResult(
                        self.editor_type.clone(),
                        self.caller.clone(),
                        r,
                        self.fetch_result_internal_selector.clone(),
                    )),
                    Err(_) => Some(Msg::None),
                }
            }
            Some(Msg::EditorPopupNext) => {
                self.next_component();
                Some(Msg::None)
            }
            m => m,
        }
    }
}

impl MockComponent for EditorPopup {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let mut total_height = 0u16;
        for (_, editor_type) in &self.components {
            let component_height = match editor_type {
                EditorType::Multiline => 10,
                EditorType::Oneline => 3,
            };
            total_height += component_height;
        }

        if self.components.len() > 1 {
            total_height += 2;
        }

        let content_width = 60u16.min(area.width);

        let actual_height = total_height.min(area.height);
        let actual_width = content_width;

        let x = area.x + (area.width.saturating_sub(actual_width)) / 2;
        let y = area.y + (area.height.saturating_sub(actual_height)) / 2;

        let popup_area = ratatui::layout::Rect {
            x,
            y,
            width: actual_width,
            height: actual_height,
        };

        frame.render_widget(Clear, popup_area);

        let inner_area = if self.components.len() > 1 {
            let block = Block::default()
                // .title(self.get_title())
                .title("Editor")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Color::White);

            frame.render_widget(block, popup_area);
            popup_area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            })
        } else {
            popup_area
        };

        let constraints = self
            .components
            .iter()
            .map(|(_, editor_type)| match editor_type {
                EditorType::Multiline => Constraint::Length(10),
                EditorType::Oneline => Constraint::Length(3),
            });
        let chunks = Layout::default()
            .direction(RatatuiDirection::Vertical)
            .constraints(constraints)
            .split(inner_area);

        for (index, (component, _)) in self.components.iter_mut().enumerate() {
            component.view(frame, chunks[index]);
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        let (component, _) = self.components.get(self.selected_component_index).unwrap();
        component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match (attr, &value) {
            (Attribute::Focus, AttrValue::Flag(_)) => {
                self.components
                    .iter_mut()
                    .for_each(|c| c.0.attr(Attribute::Focus, AttrValue::Flag(false)));
            }
            _ => (),
        };
        let (component, _) = self
            .components
            .get_mut(self.selected_component_index)
            .unwrap();
        component.attr(attr, value)
    }

    fn state(&self) -> tuirealm::State {
        let (component, _) = self.components.get(self.selected_component_index).unwrap();
        component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        let (component, _) = self
            .components
            .get_mut(self.selected_component_index)
            .unwrap();
        component.perform(cmd)
    }
}
