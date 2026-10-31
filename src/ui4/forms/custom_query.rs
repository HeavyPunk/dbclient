use iocraft::{
    AnyElement, Color, Hooks, JustifyContent, KeyCode, KeyEventKind, KeyModifiers, Props,
    component,
    components::{Text, TextInput, View},
    element,
    hooks::{UseComponentRect, UseState},
};

use crate::ui4::{app_state::QueryResultCmd, control::UseHotkeys};

#[derive(Default, Props)]
pub struct CustomQueryProps {
    pub cmd_pipe: Option<tokio::sync::mpsc::Sender<QueryResultCmd>>,
}

#[component]
pub fn CustomQuery(props: &CustomQueryProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    // NOTE: layout_ready is used to prevent invalid TextInput render on the first time render.
    // Multiline mode should be enabled only after fully built widget layout
    let mut layout_ready = false;
    if hooks.use_component_rect().is_some() {
        layout_ready = true;
    }
    let mut is_focused = hooks.use_state(|| false);
    let mut query_content = hooks.use_state(|| String::default());

    let cmd_pipe = props.cmd_pipe.clone();
    hooks.use_hotkeys(
        || true,
        move |reg| {
            {
                let cmd_pipe = cmd_pipe.clone();
                let _ = reg.register(
                    &[(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Press)],
                    move |_| {
                        if !is_focused.get() {
                            if let Some(ch) = &cmd_pipe {
                                if let Err(e) = tokio::task::block_in_place(|| {
                                    ch.blocking_send(QueryResultCmd::ExecuteRawQuery(
                                        query_content.read().clone(),
                                    ))
                                }) {
                                    log::warn!("{e}")
                                }
                            }
                        }
                        Ok(())
                    },
                );
            }
            let _ = reg.register(
                &[(KeyCode::Char('i'), KeyModifiers::NONE, KeyEventKind::Press)],
                move |_| {
                    if !is_focused.get() {
                        is_focused.set(true);
                    }
                    Ok(())
                },
            );
            {
                let cmd_pipe = cmd_pipe.clone();
                let _ = reg.register(
                    &[(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Press)],
                    move |_| {
                        if is_focused.get() {
                            is_focused.set(false);
                        } else {
                            if let Some(ch) = &cmd_pipe {
                                if let Err(e) = tokio::task::block_in_place(|| {
                                    ch.blocking_send(QueryResultCmd::ClosePopup)
                                }) {
                                    log::warn!("{e}")
                                }
                            }
                        }
                        Ok(())
                    },
                );
            }
        },
    );

    element! {
        View(
            width: 100pct,
            height: 100pct,
            flex_direction: iocraft::FlexDirection::Column,
            border_style: iocraft::components::BorderStyle::Round,
            border_color: Color::Yellow
        ) {
            View (
                justify_content: JustifyContent::Center
            ) {
                Text(content: "Custom query")
            }
            View(
                width: 100pct,
            ) {
                TextInput(
                    has_focus: is_focused.get(),
                    multiline: layout_ready,
                    auto_grow: layout_ready,
                    value: query_content.read().clone(),
                    on_change: move |new_value| {
                        query_content.set(new_value);
                    }
                )
            }
        }
    }
}
