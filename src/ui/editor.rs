use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};


pub fn edit<F>(mut init_value: String, mut render: F) where F: FnMut(String) -> () {
    loop {
        match event::read().expect("failed to read events") {
            event::Event::Key(key_event) => {

                match key_event {
                    KeyEvent { code: KeyCode::Char(symb), modifiers: _, kind: _, state: _ } => {
                        init_value.push(symb);
                        render(init_value.clone());
                    },
                    KeyEvent { code: KeyCode::Backspace, modifiers: _, kind: _, state: _ } => {
                        init_value.pop();
                        render(init_value.clone());
                    },
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: _, state: _ } | KeyEvent { code: KeyCode::Enter, modifiers: _, kind: _, state: _ } => {
                        break;
                    },
                    _ => {}
                };

            },
            _ => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::edit;

    fn should() {
        let mut s = String::from("");
        edit(s.clone(), |update| s = update);
    }
}

