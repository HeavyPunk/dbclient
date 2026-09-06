use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use iocraft::{
    component,
    components::{ScrollView, ScrollViewHandle, Text, View},
    element,
    hooks::{UseRef, UseState, UseTerminalEvents},
    AnyElement, Color, FlexDirection, Hooks, KeyCode, KeyEvent, KeyEventKind, Props, TerminalEvent,
};
use tokio::sync::Mutex as TokioMutex;

#[derive(Clone, Debug)]
pub struct FileTreeContext<T> {
    pub path: Vec<String>,
    pub descriptor: T,
}

#[derive(Clone, Default)]
pub struct FileTreeNode<T> {
    pub name: String,
    pub children: Vec<FileTreeNode<T>>,
    context: Option<T>,
    callback: Option<Arc<Mutex<Box<dyn FnMut(FileTreeContext<T>) + Send>>>>,
}

impl<T> FileTreeNode<T> {
    pub fn new(name: impl Into<String>, children: Vec<FileTreeNode<T>>) -> Self {
        Self {
            name: name.into(),
            children,
            context: None,
            callback: None,
        }
    }

    pub fn with_context(mut self, descriptor: T) -> Self {
        self.context = Some(descriptor);
        self
    }

    pub fn on_enter<F>(mut self, callback: F) -> Self
    where
        F: FnMut(FileTreeContext<T>) + Send + 'static,
    {
        self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        self
    }
}

#[derive(Clone)]
struct VisibleNode<T> {
    path: Vec<String>,
    name: String,
    depth: usize,
    has_children: bool,
    context: Option<T>,
    callback: Option<Arc<Mutex<Box<dyn FnMut(FileTreeContext<T>) + Send>>>>,
}

fn flatten_children<T: Clone>(
    nodes: &[FileTreeNode<T>],
    parent: &[String],
    depth: usize,
    expanded: &HashSet<Vec<String>>,
    output: &mut Vec<VisibleNode<T>>,
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
            context: node.context.clone(),
            callback: node.callback.clone(),
        });
        if has_children && expanded.contains(&path) {
            flatten_children(&node.children, &path, depth + 1, expanded, output);
        }
    }
}

fn flatten<T: Clone>(
    nodes: &[FileTreeNode<T>],
    expanded: &HashSet<Vec<String>>,
) -> Vec<VisibleNode<T>> {
    let mut output = Vec::new();
    flatten_children(nodes, &[], 0, expanded, &mut output);
    output
}

#[derive(Default, Props)]
pub struct FileTreeProps<T: Send + Sync> {
    pub nodes: Vec<FileTreeNode<T>>,
    pub selected_path: Arc<TokioMutex<Option<Vec<String>>>>,
}

#[component]
pub fn FileTree<T: Clone + Send + Sync + 'static>(
    props: &FileTreeProps<T>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut cursor = hooks.use_state(|| 0usize);
    let mut expanded = hooks.use_state(HashSet::<Vec<String>>::new);
    let mut scroll_handle = hooks.use_ref_default::<ScrollViewHandle>();
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

    let cursor_index = cursor.get();
    let viewport_height = scroll_handle.read().viewport_height() as usize;
    if viewport_height > 0 {
        let current_offset = scroll_handle.read().scroll_offset().max(0) as usize;
        let next_offset = if cursor_index < current_offset {
            cursor_index
        } else if cursor_index >= current_offset + viewport_height {
            cursor_index - viewport_height + 1
        } else {
            current_offset
        };

        if next_offset != current_offset {
            scroll_handle.write().scroll_to(next_offset as i32);
        }
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
            KeyCode::Enter => {
                let node = &event_visible[cursor.get()];
                if let Some(callback) = node.callback.clone() {
                    let Some(descriptor) = node.context.clone() else {
                        return;
                    };
                    if let Ok(mut callback) = callback.lock() {
                        callback(FileTreeContext {
                            path: node.path.clone(),
                            descriptor,
                        });
                    }
                }
            }
            KeyCode::Char(' ') => {
                let node = &event_visible[cursor.get()];
                if node.has_children {
                    let path = node.path.clone();
                    let mut next = expanded.read().clone();
                    if !next.insert(path) {
                        next.remove(&node.path);
                    }
                    expanded.set(next);
                }
            }
            _ => {}
        }
    });

    element! {
        View(width: 100pct, height: 100pct, flex_direction: FlexDirection::Column) {
            ScrollView(handle: Some(scroll_handle), scrollbar: Some(true), keyboard_scroll: Some(false)) {
                View(flex_direction: FlexDirection::Column) {
                    #(visible.iter().enumerate().map(|(index, node)| element! {
                        View(background_color: if index == cursor.get() { Some(Color::Blue) } else { None }) {
                            Text(content: format!("{}{} {}", "  ".repeat(node.depth), if node.has_children { "▸" } else { " " }, node.name), color: if index == cursor.get() { Some(Color::Black) } else { None })
                        }
                    }))
                }
            }
        }
    }
}
