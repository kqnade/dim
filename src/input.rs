#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Delete,
    Escape,
    Char(char),
    F(u8),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Default::default()
        }
    }

    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key {
        code: KeyCode,
        modifiers: Modifiers,
    },
    Text(String),
    Paste(String),
    Resize {
        rows: u16,
        cols: u16,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_char() {
        let code = KeyCode::Char('a');
        assert_eq!(code, KeyCode::Char('a'));
        assert_ne!(code, KeyCode::Char('b'));
    }

    #[test]
    fn test_modifiers_none() {
        let m = Modifiers::none();
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.super_key);
    }

    #[test]
    fn test_modifiers_ctrl() {
        let m = Modifiers::ctrl();
        assert!(m.ctrl);
        assert!(!m.shift);
    }

    #[test]
    fn test_input_event_key() {
        let ev = InputEvent::Key {
            code: KeyCode::Enter,
            modifiers: Modifiers::none(),
        };
        assert_eq!(ev, InputEvent::Key { code: KeyCode::Enter, modifiers: Modifiers::none() });
    }

    #[test]
    fn test_input_event_text() {
        let ev = InputEvent::Text("hello".to_string());
        assert_eq!(ev, InputEvent::Text("hello".to_string()));
    }

    #[test]
    fn test_input_event_paste() {
        let ev = InputEvent::Paste("pasted content".to_string());
        assert_eq!(ev, InputEvent::Paste("pasted content".to_string()));
    }

    #[test]
    fn test_input_event_resize() {
        let ev = InputEvent::Resize { rows: 24, cols: 80 };
        assert_eq!(ev, InputEvent::Resize { rows: 24, cols: 80 });
    }
}
