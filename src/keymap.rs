use crate::command::Command;
use crate::editor_state::EditorMode;
use crate::input::{InputEvent, KeyCode};

#[derive(Debug, Clone)]
pub struct Keymap {
    /// Stack of pending prefix keys. Empty means idle.
    prefix_stack: Vec<InputEvent>,
    /// Pending single-key for double-tap sequences (e.g. qq, gg).
    pending_key: Option<char>,
}

impl Keymap {
    pub fn new() -> Self {
        Self {
            prefix_stack: Vec::new(),
            pending_key: None,
        }
    }

    /// Clears any pending prefix state (e.g. on mode switch or cancel).
    pub fn clear_prefix(&mut self) {
        self.prefix_stack.clear();
        self.pending_key = None;
    }

    /// Maps an input event to a command based on the current editor mode.
    /// Returns `None` if the event is consumed as part of a prefix sequence
    /// or should be handled as direct text input.
    pub fn handle(&mut self, event: &InputEvent, mode: EditorMode) -> Option<Command> {
        // If we are in the middle of a prefix sequence, handle it first.
        if !self.prefix_stack.is_empty() {
            return self.handle_prefix(event, mode);
        }

        match mode {
            EditorMode::Normal => self.handle_normal(event),
            EditorMode::Command => self.handle_command(event),
            EditorMode::Search => self.handle_search(event),
            EditorMode::Insert => None,
        }
    }

    fn handle_prefix(&mut self, event: &InputEvent, _mode: EditorMode) -> Option<Command> {
        // Currently only single-level prefix is supported.
        let first = self.prefix_stack.first()?;

        match first {
            // Ctrl+X prefix (Emacs-like)
            InputEvent::Key {
                code: KeyCode::Char('x'),
                modifiers,
            } if modifiers.ctrl => match event {
                InputEvent::Key {
                    code: KeyCode::Char('s'),
                    modifiers,
                } if modifiers.ctrl => {
                    self.prefix_stack.clear();
                    Some(Command::SaveFile)
                }
                InputEvent::Key {
                    code: KeyCode::Char('c'),
                    modifiers,
                } if modifiers.ctrl => {
                    self.prefix_stack.clear();
                    Some(Command::Quit)
                }
                InputEvent::Key {
                    code: KeyCode::Char('f'),
                    modifiers,
                } if modifiers.ctrl => {
                    self.prefix_stack.clear();
                    Some(Command::EnterCommandMode)
                }
                InputEvent::Key {
                    code: KeyCode::Char('x'),
                    modifiers,
                } if !modifiers.ctrl => {
                    self.prefix_stack.clear();
                    Some(Command::EnterCommandMode)
                }
                // Cancel prefix with Ctrl+G or Escape
                InputEvent::Key {
                    code: KeyCode::Char('g'),
                    modifiers,
                } if modifiers.ctrl => {
                    self.prefix_stack.clear();
                    None
                }
                InputEvent::Key {
                    code: KeyCode::Escape,
                    ..
                } => {
                    self.prefix_stack.clear();
                    None
                }
                // Unknown key cancels prefix and is re-processed as normal input
                _ => {
                    self.prefix_stack.clear();
                    // Re-dispatch as normal input without prefix
                    self.handle_normal(event)
                }
            },
            _ => {
                self.prefix_stack.clear();
                None
            }
        }
    }

    fn handle_normal(&mut self, event: &InputEvent) -> Option<Command> {
        match event {
            // Prefix triggers
            InputEvent::Key {
                code: KeyCode::Char('x'),
                modifiers,
            } if modifiers.ctrl => {
                self.prefix_stack.push(event.clone());
                None
            }
            // Direct commands
            InputEvent::Key {
                code: KeyCode::Left,
                ..
            } => Some(Command::MoveLeft),
            InputEvent::Key {
                code: KeyCode::Right,
                ..
            } => Some(Command::MoveRight),
            InputEvent::Key {
                code: KeyCode::Up,
                ..
            } => Some(Command::MoveUp),
            InputEvent::Key {
                code: KeyCode::Down,
                ..
            } => Some(Command::MoveDown),
            InputEvent::Key {
                code: KeyCode::Home,
                ..
            } => Some(Command::MoveLineStart),
            InputEvent::Key {
                code: KeyCode::End,
                ..
            } => Some(Command::MoveLineEnd),
            InputEvent::Text(text) => {
                let ch = text.chars().next()?;

                // Handle double-tap and multi-char sequences
                match ch {
                    'q' | 'g' => {
                        if let Some(pending) = self.pending_key {
                            if pending == ch {
                                // Double-tap detected
                                self.pending_key = None;
                                return match ch {
                                    'q' => Some(Command::Quit),
                                    'g' => Some(Command::MoveFileStart),
                                    _ => None,
                                };
                            }
                        }
                        // Set as pending and consume
                        self.pending_key = Some(ch);
                        None
                    }
                    _ => {
                        // Any other key cancels pending state
                        self.pending_key = None;
                        self.handle_text_normal(text)
                    }
                }
            }
            InputEvent::Key {
                code: KeyCode::Char('r'),
                modifiers,
            } if modifiers.ctrl => Some(Command::Redo),
            InputEvent::Paste(_data) => Some(Command::PasteAfter),
            _ => {
                self.pending_key = None;
                None
            }
        }
    }

    fn handle_text_normal(&mut self, text: &str) -> Option<Command> {
        match text {
            // Colemak-DH: m/n/e/i = h/j/k/l
            "m" => Some(Command::MoveLeft),
            "n" => Some(Command::MoveDown),
            "e" => Some(Command::MoveUp),
            "i" => Some(Command::MoveRight),
            // Page scroll
            "j" => Some(Command::PageUp),
            "k" => Some(Command::PageDown),
            // Line movement
            "0" => Some(Command::MoveLineStart),
            "$" => Some(Command::MoveLineEnd),
            // Word movement
            "l" => Some(Command::MoveWordBackward),
            "u" => Some(Command::MoveWordForward),
            // Colemak-DH: s = insert, t = append
            "s" => Some(Command::EnterInsertMode),
            "t" => Some(Command::EnterAppendMode),
            // Search mode
            "/" => Some(Command::EnterSearchMode),
            // Selection
            "a" => Some(Command::VisualMode),
            "x" => Some(Command::SelectLine),
            // Editing
            "d" => Some(Command::DeleteSelection),
            "c" => Some(Command::YankSelection),
            "v" => Some(Command::PasteAfter),
            "V" => Some(Command::PasteBefore),
            // Undo/redo
            "z" => Some(Command::Undo),
            "Z" => Some(Command::Redo),
            // File end
            "G" => Some(Command::MoveFileEnd),
            // Toggle case
            "~" => Some(Command::ToggleCase),
            // Indent / unindent
            ">" => Some(Command::IndentSelection),
            "<" => Some(Command::UnindentSelection),
            // Jump matching pair
            "%" => Some(Command::JumpMatchingPair),
            // Change (reserved for now, same as before)
            "w" => Some(Command::ChangeSelection),
            _ => None,
        }
    }

    fn handle_command(&mut self, event: &InputEvent) -> Option<Command> {
        match event {
            InputEvent::Key {
                code: KeyCode::Escape,
                ..
            } => Some(Command::EnterNormalMode),
            InputEvent::Key {
                code: KeyCode::Enter,
                ..
            } => Some(Command::EnterNormalMode), // App will execute before switching
            _ => None,
        }
    }

    fn handle_search(&mut self, event: &InputEvent) -> Option<Command> {
        match event {
            InputEvent::Key {
                code: KeyCode::Escape,
                ..
            } => Some(Command::EnterNormalMode),
            InputEvent::Key {
                code: KeyCode::Enter,
                ..
            } => Some(Command::EnterNormalMode),
            _ => None,
        }
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Modifiers;

    #[test]
    fn test_normal_move_left() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Key {
            code: KeyCode::Left,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveLeft)
        );
    }

    #[test]
    fn test_normal_move_right() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Key {
            code: KeyCode::Right,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveRight)
        );
    }

    #[test]
    fn test_normal_colemak_movement() {
        let mut keymap = Keymap::new();
        assert_eq!(
            keymap.handle(&InputEvent::Text("m".to_string()), EditorMode::Normal),
            Some(Command::MoveLeft)
        );
        assert_eq!(
            keymap.handle(&InputEvent::Text("n".to_string()), EditorMode::Normal),
            Some(Command::MoveDown)
        );
        assert_eq!(
            keymap.handle(&InputEvent::Text("e".to_string()), EditorMode::Normal),
            Some(Command::MoveUp)
        );
        assert_eq!(
            keymap.handle(&InputEvent::Text("i".to_string()), EditorMode::Normal),
            Some(Command::MoveRight)
        );
    }

    #[test]
    fn test_normal_enter_insert() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("s".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::EnterInsertMode)
        );
    }

    #[test]
    fn test_normal_enter_append() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("t".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::EnterAppendMode)
        );
    }

    #[test]
    fn test_normal_line_start() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("0".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveLineStart)
        );
    }

    #[test]
    fn test_normal_line_end() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("$".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveLineEnd)
        );
    }

    #[test]
    fn test_normal_undo() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("z".to_string());
        assert_eq!(keymap.handle(&ev, EditorMode::Normal), Some(Command::Undo));
    }

    #[test]
    fn test_normal_redo() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("Z".to_string());
        assert_eq!(keymap.handle(&ev, EditorMode::Normal), Some(Command::Redo));
    }

    #[test]
    fn test_normal_yank() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("c".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::YankSelection)
        );
    }

    #[test]
    fn test_normal_paste() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("v".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::PasteAfter)
        );
    }

    #[test]
    fn test_normal_paste_before() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("V".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::PasteBefore)
        );
    }

    #[test]
    fn test_normal_change() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("w".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::ChangeSelection)
        );
    }

    #[test]
    fn test_insert_returns_none() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("a".to_string());
        assert_eq!(keymap.handle(&ev, EditorMode::Insert), None);
    }

    #[test]
    fn test_command_escape() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Key {
            code: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            keymap.handle(&ev, EditorMode::Command),
            Some(Command::EnterNormalMode)
        );
    }

    #[test]
    fn test_search_escape() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Key {
            code: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            keymap.handle(&ev, EditorMode::Search),
            Some(Command::EnterNormalMode)
        );
    }

    #[test]
    fn test_normal_unknown_key() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("q".to_string());
        assert_eq!(keymap.handle(&ev, EditorMode::Normal), None);
    }

    // --- Prefix key tests ---

    #[test]
    fn test_prefix_cx_cs_save() {
        let mut keymap = Keymap::new();
        // Ctrl+X enters prefix state
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cx, EditorMode::Normal), None);
        // Ctrl+S executes SaveFile
        let cs = InputEvent::Key {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(
            keymap.handle(&cs, EditorMode::Normal),
            Some(Command::SaveFile)
        );
    }

    #[test]
    fn test_prefix_cx_cc_quit() {
        let mut keymap = Keymap::new();
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cx, EditorMode::Normal), None);
        let cc = InputEvent::Key {
            code: KeyCode::Char('c'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(
            keymap.handle(&cc, EditorMode::Normal),
            Some(Command::Quit)
        );
    }

    #[test]
    fn test_prefix_cx_x_command_mode() {
        let mut keymap = Keymap::new();
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cx, EditorMode::Normal), None);
        let x = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            keymap.handle(&x, EditorMode::Normal),
            Some(Command::EnterCommandMode)
        );
    }

    #[test]
    fn test_prefix_cancel_cg() {
        let mut keymap = Keymap::new();
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cx, EditorMode::Normal), None);
        let cg = InputEvent::Key {
            code: KeyCode::Char('g'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cg, EditorMode::Normal), None);
        // After cancel, normal keys work again
        let ev = InputEvent::Text("m".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveLeft)
        );
    }

    #[test]
    fn test_prefix_cancel_escape() {
        let mut keymap = Keymap::new();
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cx, EditorMode::Normal), None);
        let esc = InputEvent::Key {
            code: KeyCode::Escape,
            modifiers: Modifiers::none(),
        };
        assert_eq!(keymap.handle(&esc, EditorMode::Normal), None);
    }

    #[test]
    fn test_prefix_unknown_then_redispatch() {
        let mut keymap = Keymap::new();
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(keymap.handle(&cx, EditorMode::Normal), None);
        // Unknown key 'm' cancels prefix and is re-dispatched
        let ev = InputEvent::Text("m".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveLeft)
        );
    }

    #[test]
    fn test_prefix_does_not_affect_other_modes() {
        let mut keymap = Keymap::new();
        let cx = InputEvent::Key {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::ctrl(),
        };
        // In Insert mode, Ctrl+X is not a prefix, it's just a key event
        assert_eq!(keymap.handle(&cx, EditorMode::Insert), None);
        // Stack should remain empty
        assert!(keymap.prefix_stack.is_empty());
    }

    #[test]
    fn test_normal_visual_mode() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("a".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::VisualMode)
        );
    }

    #[test]
    fn test_normal_select_line() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("x".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::SelectLine)
        );
    }

    #[test]
    fn test_normal_page_up() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("j".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::PageUp)
        );
    }

    #[test]
    fn test_normal_page_down() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("k".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::PageDown)
        );
    }

    #[test]
    fn test_normal_word_forward() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("u".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveWordForward)
        );
    }

    #[test]
    fn test_normal_word_backward() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("l".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveWordBackward)
        );
    }

    #[test]
    fn test_normal_file_start() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("g".to_string());
        // First 'g' is consumed as pending
        assert_eq!(keymap.handle(&ev, EditorMode::Normal), None);
        // Second 'g' executes MoveFileStart
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveFileStart)
        );
    }

    #[test]
    fn test_normal_file_end() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("G".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::MoveFileEnd)
        );
    }

    #[test]
    fn test_normal_toggle_case() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("~".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::ToggleCase)
        );
    }

    #[test]
    fn test_normal_indent() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text(">".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::IndentSelection)
        );
    }

    #[test]
    fn test_normal_unindent() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("<".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::UnindentSelection)
        );
    }

    #[test]
    fn test_normal_jump_matching_pair() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("%".to_string());
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::JumpMatchingPair)
        );
    }

    #[test]
    fn test_normal_quit_double_tap() {
        let mut keymap = Keymap::new();
        let ev = InputEvent::Text("q".to_string());
        // First 'q' is consumed as pending
        assert_eq!(keymap.handle(&ev, EditorMode::Normal), None);
        // Second 'q' executes quit
        assert_eq!(
            keymap.handle(&ev, EditorMode::Normal),
            Some(Command::Quit)
        );
    }
}
