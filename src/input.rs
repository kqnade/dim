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

pub fn parse_crossterm_event(event: crossterm::event::Event) -> Option<InputEvent> {
    use crossterm::event::{Event, KeyCode as CKeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    match event {
        Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
            if kind != KeyEventKind::Press {
                return None;
            }

            let has_ctrl = modifiers.contains(KeyModifiers::CONTROL);
            let has_alt = modifiers.contains(KeyModifiers::ALT);
            let has_super = modifiers.contains(KeyModifiers::SUPER);

            match code {
                CKeyCode::Char(c) if !has_ctrl && !has_alt && !has_super => {
                    Some(InputEvent::Text(c.to_string()))
                }
                _ => {
                    let our_code = match code {
                        CKeyCode::Backspace => KeyCode::Backspace,
                        CKeyCode::Enter => KeyCode::Enter,
                        CKeyCode::Left => KeyCode::Left,
                        CKeyCode::Right => KeyCode::Right,
                        CKeyCode::Up => KeyCode::Up,
                        CKeyCode::Down => KeyCode::Down,
                        CKeyCode::Home => KeyCode::Home,
                        CKeyCode::End => KeyCode::End,
                        CKeyCode::PageUp => KeyCode::PageUp,
                        CKeyCode::PageDown => KeyCode::PageDown,
                        CKeyCode::Tab => KeyCode::Tab,
                        CKeyCode::Delete => KeyCode::Delete,
                        CKeyCode::Esc => KeyCode::Escape,
                        CKeyCode::Char(c) => KeyCode::Char(c),
                        CKeyCode::F(n) => KeyCode::F(n),
                        CKeyCode::Null => KeyCode::Null,
                        _ => return None,
                    };

                    let our_mods = Modifiers {
                        shift: modifiers.contains(KeyModifiers::SHIFT),
                        ctrl: has_ctrl,
                        alt: has_alt,
                        super_key: has_super,
                    };

                    Some(InputEvent::Key {
                        code: our_code,
                        modifiers: our_mods,
                    })
                }
            }
        }
        Event::Resize(cols, rows) => {
            Some(InputEvent::Resize { rows, cols })
        }
        Event::Paste(data) => {
            Some(InputEvent::Paste(data))
        }
        _ => None,
    }
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

    #[test]
    fn test_parse_crossterm_char_text() {
        use crossterm::event::{KeyCode as CKeyCode, KeyEvent, KeyModifiers};
        let cevent = crossterm::event::Event::Key(KeyEvent::from(CKeyCode::Char('a')));
        let ev = parse_crossterm_event(cevent).unwrap();
        assert_eq!(ev, InputEvent::Text("a".to_string()));
    }

    #[test]
    fn test_parse_crossterm_ctrl_char_key() {
        use crossterm::event::{KeyCode as CKeyCode, KeyEvent, KeyModifiers};
        let cevent = crossterm::event::Event::Key(KeyEvent {
            code: CKeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        let ev = parse_crossterm_event(cevent).unwrap();
        assert_eq!(
            ev,
            InputEvent::Key {
                code: KeyCode::Char('c'),
                modifiers: Modifiers::ctrl(),
            }
        );
    }

    #[test]
    fn test_parse_crossterm_escape() {
        use crossterm::event::{KeyCode as CKeyCode, KeyEvent, KeyModifiers};
        let cevent = crossterm::event::Event::Key(KeyEvent::from(CKeyCode::Esc));
        let ev = parse_crossterm_event(cevent).unwrap();
        assert_eq!(
            ev,
            InputEvent::Key {
                code: KeyCode::Escape,
                modifiers: Modifiers::none(),
            }
        );
    }

    #[test]
    fn test_parse_crossterm_resize() {
        let cevent = crossterm::event::Event::Resize(80, 24);
        let ev = parse_crossterm_event(cevent).unwrap();
        assert_eq!(ev, InputEvent::Resize { rows: 24, cols: 80 });
    }

    #[test]
    fn test_parse_crossterm_paste() {
        let cevent = crossterm::event::Event::Paste("hello".to_string());
        let ev = parse_crossterm_event(cevent).unwrap();
        assert_eq!(ev, InputEvent::Paste("hello".to_string()));
    }

    #[test]
    fn test_parse_crossterm_release_ignored() {
        use crossterm::event::{KeyCode as CKeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        let cevent = crossterm::event::Event::Key(KeyEvent {
            code: CKeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!(parse_crossterm_event(cevent), None);
    }
}
