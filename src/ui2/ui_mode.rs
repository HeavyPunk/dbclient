use string_cmd::StringEditor;

#[derive(Clone)]
pub enum UserMode {
    Normal,
    Insert,
    SearchInput,
    Search(String, usize),
    Command,
}

