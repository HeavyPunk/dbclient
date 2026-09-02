use std::{collections::HashSet, sync::Arc};

use iocraft::{
    component,
    components::{Text, View},
    element,
    hooks::{UseState, UseTerminalEvents},
    AnyElement, Color, FlexDirection, Hooks, KeyCode, KeyEvent, KeyEventKind, Props, TerminalEvent,
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileTreeNode {
    pub name: String,
    pub children: Vec<FileTreeNode>,
}

impl FileTreeNode {
    pub fn new(name: impl Into<String>, children: Vec<FileTreeNode>) -> Self {
        Self {
            name: name.into(),
            children,
        }
    }
}

#[derive(Clone, Debug)]
struct VisibleNode {
    path: Vec<String>,
    name: String,
    depth: usize,
    has_children: bool,
}

fn flatten_children(
    nodes: &[FileTreeNode],
    parent: &[String],
    depth: usize,
    expanded: &HashSet<Vec<String>>,
    output: &mut Vec<VisibleNode>,
) {
    for node in nodes {
        let mut path = parent.to_vec();
        path.push(node.name.clone());
        let has_children = !node.children.is_empty();
        output.push(VisibleNode {
            path: path.clone(),
            name: node.name.clone(),
            depth,
            has_children,
        });
        if has_children && expanded.contains(&path) {
            flatten_children(&node.children, &path, depth + 1, expanded, output);
        }
    }
}

fn flatten(nodes: &[FileTreeNode], expanded: &HashSet<Vec<String>>) -> Vec<VisibleNode> {
    let mut output = Vec::new();
    flatten_children(nodes, &[], 0, expanded, &mut output);
    output
}

#[derive(Default, Props)]
pub struct FileTreeProps {
    pub nodes: Vec<FileTreeNode>,
    pub selected_path: Arc<Mutex<Option<Vec<String>>>>,
}

#[component]
pub fn FileTree(props: &FileTreeProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut cursor = hooks.use_state(|| 0usize);
    let mut expanded = hooks.use_state(HashSet::<Vec<String>>::new);
    let nodes = props.nodes.clone();
    let selected_path = props.selected_path.clone();
    let visible = flatten(&nodes, &expanded.read());

    if visible.is_empty() {
        cursor.set(0);
    } else if cursor.get() >= visible.len() {
        cursor.set(visible.len() - 1);
    }

    if let Some(path) = visible.get(cursor.get()).map(|node| node.path.clone()) {
        tokio::task::block_in_place(|| *selected_path.blocking_lock() = Some(path));
    }

    let event_visible = visible.clone();
    hooks.use_local_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release || event_visible.is_empty() {
            return;
        }

        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                cursor.set((cursor.get() + 1).min(event_visible.len() - 1))
            }
            KeyCode::Up | KeyCode::Char('k') => cursor.set(cursor.get().saturating_sub(1)),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let node = &event_visible[cursor.get()];
                let path = node.path.clone();
                if node.has_children {
                    let mut next = expanded.read().clone();
                    if !next.insert(path.clone()) {
                        next.remove(&path);
                    }
                    expanded.set(next);
                }
                let selected_path = selected_path.clone();
                tokio::task::block_in_place(|| *selected_path.blocking_lock() = Some(path));
            }
            _ => {}
        }
    });

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(visible.iter().enumerate().map(|(index, node)| element! {
                View(background_color: if index == cursor.get() { Some(Color::Blue) } else { None }) {
                    Text(content: format!("{}{} {}", "  ".repeat(node.depth), if node.has_children { "▸" } else { " " }, node.name), color: if index == cursor.get() { Some(Color::White) } else { None })
                }
            }))
        }
    }
}
