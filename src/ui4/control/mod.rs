use std::{pin::pin, task::Poll};

use dbclient::hotkey_manager::HotKeyManager;
use futures::Stream;
use iocraft::{Hook, Hooks, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, TerminalEvents};

pub trait UseHotkeys<'a> {
    fn use_hotkeys<Registerer, FocusGetter>(
        &mut self,
        is_focused: FocusGetter,
        registerer: Registerer,
    ) where
        Registerer:
            FnMut(&mut HotKeyManager<(KeyCode, KeyModifiers, KeyEventKind), (), ()>) + Send + 'a,
        FocusGetter: Fn() -> bool + Send + Unpin + 'static;
}

impl<'a> UseHotkeys<'a> for Hooks<'a, '_> {
    fn use_hotkeys<Registerer, FocusGetter>(
        &mut self,
        is_focused: FocusGetter,
        mut registerer: Registerer,
    ) where
        Registerer:
            FnMut(&mut HotKeyManager<(KeyCode, KeyModifiers, KeyEventKind), (), ()>) + Send + 'a,
        FocusGetter: Fn() -> bool + Send + Unpin + 'static,
    {
        let mut hotkey_manager = HotKeyManager::new();
        registerer(&mut hotkey_manager);
        let h = self.use_hook(move || UseHotkeysImpl {
            events: None,
            hotkey_manager: hotkey_manager,
            is_focused: None,
        });
        h.is_focused = Some(is_focused);
    }
}

struct UseHotkeysImpl<FocusGetter: Fn() -> bool> {
    is_focused: Option<FocusGetter>,
    hotkey_manager: HotKeyManager<(KeyCode, KeyModifiers, KeyEventKind), (), ()>,
    events: Option<TerminalEvents>,
}

impl<FocusGetter: Fn() -> bool + Send + Unpin> Hook for UseHotkeysImpl<FocusGetter> {
    fn poll_change(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<()> {
        while let Some(Poll::Ready(Some(event))) = self
            .events
            .as_mut()
            .map(|events| pin!(events).poll_next(cx))
        {
            match event {
                iocraft::TerminalEvent::Key(KeyEvent {
                    code,
                    kind,
                    modifiers,
                    ..
                }) => {
                    if kind == KeyEventKind::Release {
                        continue;
                    }
                    if let Some(is_focused) = &self.is_focused {
                        if !is_focused() {
                            self.hotkey_manager.clear_buffer();
                            return Poll::Pending;
                        }
                    }
                    let _ = self.hotkey_manager.push((code, modifiers, kind));
                    // TODO: log debug if combination doesn't match
                    let _ = self.hotkey_manager.invoke_if_matched(&mut ());
                }
                _ => {}
            }
        }
        Poll::Pending
    }

    fn post_component_update(&mut self, updater: &mut iocraft::prelude::ComponentUpdater) {
        if self.events.is_none() {
            self.events = updater.terminal_events();
        }
    }
}
