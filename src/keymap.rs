use crate::command::Command;
use crate::editor_state::EditorMode;
use crate::input::{InputEvent, KeyCode};

pub struct Keymap;

impl Keymap {
    /// Maps an input event to a command based on the current editor mode.
    /// Returns `None` if the event should be handled as direct text input
    /// (e.g., typing in Insert or Command mode).
    pub fn handle(event: &InputEvent, mode: EditorMode) -> Option<Command> {
        match mode {
            EditorMode::Normal => Self::handle_normal(event),
            EditorMode::Command => Self::handle_command(event),
            EditorMode::Search => Self::handle_search(event),
            EditorMode::Insert => None,
        }
    }

    fn handle_normal(event: &InputEvent) -> Option<Command> {
        match event {
            InputEvent::Key { code: KeyCode::Left, .. } => Some(Command::MoveLeft),
            InputEvent::Key { code: KeyCode::Right, .. } => Some(Command::MoveRight),
            InputEvent::Key { code: KeyCode::Up, .. } => Some(Command::MoveUp),
            InputEvent::Key { code: KeyCode::Down, .. } => Some(Command::MoveDown),
            InputEvent::Key { code: KeyCode::Home, .. } => Some(Command::MoveLineStart),
            InputEvent::Key { code: KeyCode::End, .. } => Some(Command::MoveLineEnd),
            InputEvent::Text(text) => match text.as_str() {
                // Colemak-DH: m/n/e/i = h/j/k/l
                "m" => Some(Command::MoveLeft),
                "n" => Some(Command::MoveDown),
                "e" => Some(Command::MoveUp),
                "i" => Some(Command::MoveRight),
                // Colemak-DH: s = insert (since i is right)
                "s" => Some(Command::EnterInsertMode),
                // Command mode: ; (Colemak-DH optimized)
                ";" => Some(Command::EnterCommandMode),
                "/" => Some(Command::EnterSearchMode),
                // Colemak-DH: x = delete, c = yank, v = paste
                "x" => Some(Command::DeleteSelection),
                "c" => Some(Command::YankSelection),
                "v" => Some(Command::PasteAfter),
                "V" => Some(Command::PasteBefore),
                // Colemak-DH: z = undo, Z = redo
                "z" => Some(Command::Undo),
                "Z" => Some(Command::Redo),
                // Colemak-DH: w = change
                "w" => Some(Command::ChangeSelection),
                _ => None,
            },
            InputEvent::Key { code: KeyCode::Char('r'), modifiers } if modifiers.ctrl => {
                Some(Command::Redo)
            }
            InputEvent::Paste(_data) => Some(Command::PasteAfter),
            _ => None,
        }
    }

    fn handle_command(event: &InputEvent) -> Option<Command> {
        match event {
            InputEvent::Key { code: KeyCode::Escape, .. } => Some(Command::EnterNormalMode),
            InputEvent::Key { code: KeyCode::Enter, .. } => Some(Command::EnterNormalMode), // App will execute before switching
            _ => None,
        }
    }

    fn handle_search(event: &InputEvent) -> Option<Command> {
        match event {
            InputEvent::Key { code: KeyCode::Escape, .. } => Some(Command::EnterNormalMode),
            InputEvent::Key { code: KeyCode::Enter, .. } => Some(Command::EnterNormalMode),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Modifiers;

    #[test]
    fn test_normal_move_left() {
        let ev = InputEvent::Key {
            code: KeyCode::Left,
            modifiers: Modifiers::none(),
        };
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::MoveLeft));
    }

    #[test]
    fn test_normal_move_right() {
        let ev = InputEvent::Key {
            code: KeyCode::Right,
            modifiers: Modifiers::none(),
        };
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::MoveRight));
    }

    #[test]
    fn test_normal_colemak_movement() {
        assert_eq!(Keymap::handle(&InputEvent::Text("m".to_string()), EditorMode::Normal), Some(Command::MoveLeft));
        assert_eq!(Keymap::handle(&InputEvent::Text("n".to_string()), EditorMode::Normal), Some(Command::MoveDown));
        assert_eq!(Keymap::handle(&InputEvent::Text("e".to_string()), EditorMode::Normal), Some(Command::MoveUp));
        assert_eq!(Keymap::handle(&InputEvent::Text("i".to_string()), EditorMode::Normal), Some(Command::MoveRight));
    }

    #[test]
    fn test_normal_enter_insert() {
        let ev = InputEvent::Text("s".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::EnterInsertMode));
    }

    #[test]
    fn test_normal_enter_command() {
        let ev = InputEvent::Text(";".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::EnterCommandMode));
    }

    #[test]
    fn test_normal_undo() {
        let ev = InputEvent::Text("z".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::Undo));
    }

    #[test]
    fn test_normal_redo() {
        let ev = InputEvent::Text("Z".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::Redo));
    }

    #[test]
    fn test_normal_delete() {
        let ev = InputEvent::Text("x".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::DeleteSelection));
    }

    #[test]
    fn test_normal_yank() {
        let ev = InputEvent::Text("c".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::YankSelection));
    }

    #[test]
    fn test_normal_paste() {
        let ev = InputEvent::Text("v".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::PasteAfter));
    }

    #[test]
    fn test_normal_paste_before() {
        let ev = InputEvent::Text("V".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::PasteBefore));
    }

    #[test]
    fn test_normal_change() {
        let ev = InputEvent::Text("w".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), Some(Command::ChangeSelection));
    }

    #[test]
    fn test_insert_returns_none() {
        let ev = InputEvent::Text("a".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Insert), None);
    }

    #[test]
    fn test_command_escape() {
        let ev = InputEvent::Key {
            code: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            Keymap::handle(&ev, EditorMode::Command),
            Some(Command::EnterNormalMode)
        );
    }

    #[test]
    fn test_search_escape() {
        let ev = InputEvent::Key {
            code: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            Keymap::handle(&ev, EditorMode::Search),
            Some(Command::EnterNormalMode)
        );
    }

    #[test]
    fn test_normal_unknown_key() {
        let ev = InputEvent::Text("q".to_string());
        assert_eq!(Keymap::handle(&ev, EditorMode::Normal), None);
    }
}
