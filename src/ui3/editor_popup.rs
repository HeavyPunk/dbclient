use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout, Direction as RatatuiDirection};
use tuirealm::{command::{Cmd, CmdResult}, AttrValue, Attribute, Component, Event, MockComponent};

use super::{query_input::EditorInput, AppEvent, Msg};

pub enum EditorType {
    Multiline,
    Oneline,
}

pub struct EditorPopup {
    editor_type: crate::ui3::EditorType,
    components: Vec<(Box<EditorInput>, EditorType)>,
    selected_component_index: usize,
}

impl EditorPopup {
    pub fn new(editor_type: crate::ui3::EditorType) -> Self {
        let components = match editor_type {
            super::EditorType::Search => vec![
                        (Box::new(EditorInput::new("Search", "search")), EditorType::Oneline)
                    ],
            super::EditorType::Query => vec![
                        (Box::new(EditorInput::new("query", "query")), EditorType::Multiline)
                    ],
            super::EditorType::AddDbObject => vec![
                (Box::new(EditorInput::new("Root", "root")), EditorType::Oneline),
                (Box::new(EditorInput::new("Type", "type")), EditorType::Oneline),
                (Box::new(EditorInput::new("Name", "name")), EditorType::Oneline),
            ],
        };

        Self {
            editor_type,
            components,
            selected_component_index: 0
        }
    }
}

impl Component<Msg, AppEvent> for EditorPopup {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        let (component, _) = self.components.get_mut(self.selected_component_index).unwrap();
        match component.on(ev) {
            Some(Msg::EditorAccept) => {
                let editors_results: HashMap<_, _> = self.components.iter_mut().map(|c| {
                    let edit_result = match c.0.perform(Cmd::Submit) {
                        CmdResult::Submit(state) => {
                            let state_lines: Vec<String> = state.unwrap_vec()
                                .iter().map(|state_val| state_val.clone().unwrap_string()).collect();
                            state_lines
                        },
                        _ => vec![]
                    };
                    (c.0.editor_type, edit_result)
                }).collect();
                Some(Msg::EditorResult(self.editor_type.clone(), editors_results))
            },
            Some(Msg::EditorPopupNext) => {
                self.selected_component_index += 1;
                if self.selected_component_index >= self.components.len() {
                    self.selected_component_index = 0;
                }
                Some(Msg::None)
            }
            m => m
        }
    }
}

impl MockComponent for EditorPopup {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let constraints = self.components.iter().map(|(_, editor_type)| match editor_type {
            EditorType::Multiline => Constraint::Fill(1),
            EditorType::Oneline => Constraint::Max(4),
        });
        let chunks = Layout::default()
            .direction(RatatuiDirection::Vertical)
            .constraints(constraints)
            .split(area);

        for (index, (component, _)) in self.components.iter_mut().enumerate() {
            component.view(frame, chunks[index]);
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        let (component, _) = self.components.get(self.selected_component_index).unwrap();
        component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        // TODO: should be removed to make a better focus switching
        match (attr, &value) {
            (Attribute::Focus, AttrValue::Flag(_)) => {
                self.components.iter_mut().for_each(|c| c.0.attr(Attribute::Focus, AttrValue::Flag(false)));
            },
            _ => ()
        };
        let (component, _) = self.components.get_mut(self.selected_component_index).unwrap();
        component.attr(attr, value)
    }

    fn state(&self) -> tuirealm::State {
        let (component, _) = self.components.get(self.selected_component_index).unwrap();
        component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        let (component, _) = self.components.get_mut(self.selected_component_index).unwrap();
        component.perform(cmd)
    }
}
