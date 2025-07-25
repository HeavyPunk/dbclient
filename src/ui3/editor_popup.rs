use std::collections::HashMap;

use ratatui::{layout::{Constraint, Direction as RatatuiDirection, Layout}, widgets::{Block, Borders, Clear}, style::Color};
use tuirealm::{command::{Cmd, CmdResult}, AttrValue, Attribute, Component, Event, MockComponent, event::KeyEvent};

use crate::ui3::editor_simple_input::EditorSimpleInput;
use crate::ui3::query_input::EditorInput;

use super::{AppEvent, Msg};

pub trait EditorPopupWidget :Component<Msg, AppEvent> {
    fn get_content(&self) -> Vec<String>;
    fn get_editor_type(&self) -> &'static str;
}

pub enum EditorType {
    Multiline,
    Oneline,
}

pub struct EditorPopup {
    editor_type: crate::ui3::EditorType,
    components: Vec<(Box<dyn EditorPopupWidget>, EditorType)>,
    selected_component_index: usize,
}

impl EditorPopup {
    pub fn new(editor_type: crate::ui3::EditorType) -> Self {
        let components : Vec<(Box<dyn EditorPopupWidget>, EditorType)>= match editor_type {
            super::EditorType::Search => vec![
                        (Box::new(EditorSimpleInput::new("Search", "search")), EditorType::Oneline)
                    ],
            super::EditorType::Query => vec![
                        (Box::new(EditorInput::new("Query", "query")), EditorType::Multiline)
                    ],
            super::EditorType::AddDbObject => vec![
                (Box::new(EditorSimpleInput::new("Root", "root")), EditorType::Oneline),
                (Box::new(EditorSimpleInput::new("Type", "type")), EditorType::Oneline),
                (Box::new(EditorSimpleInput::new("Name", "name")), EditorType::Oneline),
            ],
        };

        let mut popup = Self {
            editor_type,
            components,
            selected_component_index: 0
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

    fn get_title(&self) -> &'static str {
        match self.editor_type {
            super::EditorType::Search => "Search",
            super::EditorType::Query => "Query Editor",
            super::EditorType::AddDbObject => "Add Database Object",
        }
    }
}

impl Component<Msg, AppEvent> for EditorPopup {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        if let Event::Keyboard(KeyEvent { code, modifiers, .. }) = &ev {
            match code {
                tuirealm::event::Key::Tab => {
                    if modifiers.contains(tuirealm::event::KeyModifiers::SHIFT) {
                        self.prev_component();
                    } else {
                        self.next_component();
                    }
                    return Some(Msg::None);
                },
                _ => {}
            }
        }

        let (component, _) = self.components.get_mut(self.selected_component_index).unwrap();
        match component.on(ev) {
            Some(Msg::EditorAccept) => {
                let editors_results: HashMap<_, _> = self.components.iter().map(|c| {
                    let content = c.0.get_content();
                    (c.0.get_editor_type(), content)
                }).collect();
                Some(Msg::EditorResult(self.editor_type.clone(), editors_results))
            },
            Some(Msg::EditorPopupNext) => {
                self.next_component();
                Some(Msg::None)
            }
            m => m
        }
    }
}

impl MockComponent for EditorPopup {
    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(self.get_title())
            .borders(Borders::ALL)
            .border_style(Color::White);
        
        frame.render_widget(block, area);
        
        let inner_area = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });

        let constraints = self.components.iter().map(|(_, editor_type)| match editor_type {
            EditorType::Multiline => Constraint::Fill(1),
            EditorType::Oneline => Constraint::Max(3),
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
