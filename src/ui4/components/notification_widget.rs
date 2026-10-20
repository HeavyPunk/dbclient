use std::time::{Duration, SystemTime};

use dbclient::virtual_logger::Notification;
use iocraft::{
    component,
    components::{Text, View},
    element,
    hooks::{UseFuture, UseState},
    AnyElement, Color, Hooks, Props,
};
use tokio::sync::mpsc::Receiver;

#[derive(Clone)]
struct Record {
    notification: Notification,
    appered_at: SystemTime,
}

#[derive(Default, Props)]
pub struct NotificationWidgetProps {
    pub notification_receiver: Option<Receiver<Notification>>,
}

#[component]
pub fn NotificationWidget(
    props: &mut NotificationWidgetProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut messages = hooks.use_state(|| Vec::<Record>::new());
    let notification_receiver = props.notification_receiver.take();

    hooks.use_future(async move {
        let Some(mut notification_receiver) = notification_receiver else {
            log::warn!(target: "notification_widget", "Notification pipe is not configured, notification widget is disabled");
            return;
        };
        loop {
            match notification_receiver.recv().await {
                Some(notification) => {
                    messages.write().push(Record {
                        notification,
                        appered_at: SystemTime::now()
                    });
                },
                None => {
                    log::info!(target: "notification_widget", "Notifications pipe is closed");
                    break;
                },
            };
        }
    });

    hooks.use_future(async move {
        loop {
            messages.write().extract_if(.., |msg| {
                match msg.appered_at.elapsed() {
                    Ok(elapsed) => elapsed > Duration::from_secs(3),
                    Err(e) => {
                        log::warn!(target: "notification_widget", "Error on checking notification stack: {e}");
                        true
                    },
                }
            }).for_each(drop);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    element! {
        View (
            width: 100pct,
            height: 100pct,
            flex_direction: iocraft::FlexDirection::Column,
            background_color: Color::Black,
        ) {
            #(messages.read().iter().map(|msg| {
                element! {
                    View (
                        border_style: iocraft::components::BorderStyle::Round,
                        border_color: match msg.notification.level {
                            log::Level::Error => Color::Red,
                            log::Level::Warn => Color::Yellow,
                            log::Level::Info | log::Level::Debug | log::Level::Trace => Color::Blue,
                        },
                    ) {
                        Text(content: msg.notification.message.clone())
                    }
                }
            }))
        }
    }
}
