use std::sync::Arc;

use iocraft::{
    component,
    components::{BorderStyle, Text, TextDecoration, View},
    element,
    hooks::{UseContext, UseFuture, UseState, UseTerminalEvents, UseTerminalSize},
    AnyElement, Color, Edges, FlexDirection, Hooks, JustifyContent, KeyCode, KeyEvent,
    KeyEventKind, KeyModifiers, Props, SystemContext, TerminalEvent, Weight,
};
use tokio::sync::Mutex;

use crate::{
    core::proto::{self, connections::GetAvailableConnectionsRequest},
    ui4::{
        app_state::{AppState, Page},
        control::UseHotkeys,
    },
};

#[derive(Default, Props)]
pub struct ConnectionsListProps {
    pub connections_client: Option<Arc<dyn proto::connections::ConnectionsService + Send + Sync>>,
    pub state: Arc<Mutex<AppState>>,
}

#[component]
pub async fn ConnectionsList(
    props: &ConnectionsListProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let connections = hooks.use_state(Vec::<proto::connections::Connection>::new);
    let mut selected_row = hooks.use_state(|| 0);
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);

    if let Some(client) = props.connections_client.clone() {
        let mut connections = connections.clone();
        hooks.use_future(async move {
            if let Ok(response) = client
                .get_available_list(GetAvailableConnectionsRequest {})
                .await
            {
                connections.set(response.connections);
            }
        });
    }

    hooks.use_hotkeys(
        || true,
        move |hot_key_manager| {
            let _ = hot_key_manager.register(
                &[
                    (KeyCode::Char('g'), KeyModifiers::NONE, KeyEventKind::Press),
                    (KeyCode::Char('g'), KeyModifiers::NONE, KeyEventKind::Press),
                ],
                move |_: &mut ()| {
                    selected_row.set(0);
                    Ok(())
                },
            );
            let _ = hot_key_manager.register(
                &[(KeyCode::Char('G'), KeyModifiers::SHIFT, KeyEventKind::Press)],
                move |_: &mut ()| {
                    selected_row.set(connections.read().len().saturating_sub(1));
                    Ok(())
                },
            );
        },
    );

    if should_exit.get() {
        system.exit();
    }

    let state = props.state.clone();
    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('j') => {
                        let conn_count = connections.read().len();
                        let current_row = selected_row.get();
                        if current_row + 1 < conn_count {
                            selected_row.set(selected_row.get() + 1)
                        }
                    }
                    KeyCode::Char('k') => {
                        let current_row = selected_row.get();
                        if current_row > 0 {
                            selected_row.set(selected_row.get() - 1)
                        }
                    }
                    KeyCode::Enter => {
                        let current_row = selected_row.get();
                        let conns = connections.read();
                        let conn = conns.get(current_row);
                        if conn.is_none() {
                            //TOOD: log connection not found
                            return;
                        }
                        let mut guard = tokio::task::block_in_place(|| state.blocking_lock());
                        guard.selected_connection = Some(conn.unwrap().clone());
                        guard.selected_page = Page::QueryArea;
                    }
                    KeyCode::Esc => {
                        should_exit.set(true);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    element! {
        View(
            width: 100pct,
            height: 100pct,
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                View(width: 50pct, justify_content: JustifyContent::Start, padding_right: 2) {
                    Text(content: "Name", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 50pct, justify_content: JustifyContent::Center, padding_right: 2) {
                    Text(content: "Type", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
            }

            #(connections.read().iter().enumerate().map(|(i, connection)| element! {
                View(
                    background_color: if selected_row.get() == i {
                        Some(Color::Blue)
                    } else if i % 2 == 0 {
                        None
                    } else {
                        Some(Color::DarkGrey)
                    }
                ) {
                    View(width: 50pct, justify_content: JustifyContent::Start, padding_right: 2) {
                        Text(
                            content: connection.id.clone(),
                            color: if selected_row.get() == i || i % 2 != 0 {
                                Some(Color::Black)
                            } else {None}
                        )
                    }
                    View(width: 50pct, justify_content: JustifyContent::Center, padding_right: 2) {
                        Text(
                            content: "TODO".to_string(),
                            color: if selected_row.get() == i || i % 2 != 0 {
                                Some(Color::Black)
                            } else {None}
                        )
                    }
                }
            }))
        }
    }
}
