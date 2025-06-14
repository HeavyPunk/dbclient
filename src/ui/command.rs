use super::editor::edit;

pub enum EventKind {
    Edit,
    Enter,
}

pub fn command<F>(mut render: F)
where
    F: FnMut(String, EventKind) -> (),
{
    let mut command_buf = String::from(":");
    render(command_buf.clone(), EventKind::Edit);
    edit(command_buf.clone(), |update| {
        command_buf = update;
        render(command_buf.clone(), EventKind::Edit);
    });
    render(command_buf, EventKind::Enter);
}
