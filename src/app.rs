use crate::command::{Command, CommandRegistry};
use crate::config::Config;
use crate::editor_state::{EditorMode, EditorState};
use crate::input::{InputEvent, KeyCode};
use crate::keymap::Keymap;
use crate::renderer::Renderer;
use crate::selection::Selection;
use crate::skk::SkkAction;
use crate::terminal::Terminal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Continue,
    Quit,
    ForceQuit,
    ShowMessage(String),
}

/// Executes a single editor command against the current state.
pub fn execute_command(cmd: Command, state: &mut EditorState) -> AppAction {
    match cmd {
        Command::MoveLeft => state.move_cursor_left(),
        Command::MoveRight => state.move_cursor_right(),
        Command::MoveUp => state.move_cursor_up(),
        Command::MoveDown => state.move_cursor_down(),
        Command::MoveLineStart => state.move_head_to(Position::new(state.selection.head.line, 0)),
        Command::MoveLineEnd => {
            let line = state.selection.head.line;
            let len = state.buffer.line_len(line).unwrap_or(0);
            state.move_head_to(Position::new(line, len));
        }
        Command::MoveFileStart => state.move_head_to(Position::new(0, 0)),
        Command::MoveFileEnd => {
            let last_line = state.buffer.line_count().saturating_sub(1);
            let len = state.buffer.line_len(last_line).unwrap_or(0);
            state.move_head_to(Position::new(last_line, len));
        }
        Command::DeleteSelection => {
            if state.selection.is_empty() {
                state.delete_char();
            } else {
                state.delete_selection();
            }
        }
        Command::ChangeSelection => {
            if state.selection.is_empty() {
                state.delete_char();
            } else {
                state.delete_selection();
            }
            state.set_mode(EditorMode::Insert);
        }
        Command::YankSelection => {
            state.yank_selection();
        }
        Command::PasteBefore => {
            let text = state.yank_buffer.clone();
            if !text.is_empty() {
                state.insert_at_cursor(&text);
            }
        }
        Command::PasteAfter => {
            let text = state.yank_buffer.clone();
            if !text.is_empty() {
                state.move_cursor_right();
                state.insert_at_cursor(&text);
            }
        }
        Command::EnterNormalMode => state.set_mode(EditorMode::Normal),
        Command::EnterInsertMode => state.set_mode(EditorMode::Insert),
        Command::EnterCommandMode => {
            state.command_buffer.clear();
            state.set_mode(EditorMode::Command);
        }
        Command::EnterSearchMode => {
            state.set_mode(EditorMode::Search);
        }
        Command::SaveFile => {
            return match state.save_file() {
                Ok(()) => AppAction::ShowMessage("Saved".to_string()),
                Err(e) => AppAction::ShowMessage(format!("Error saving: {:?}", e)),
            };
        }
        Command::SaveFileAs(ref path) => {
            return match state.save_file_as(std::path::Path::new(path)) {
                Ok(()) => AppAction::ShowMessage("Saved".to_string()),
                Err(e) => AppAction::ShowMessage(format!("Error saving: {:?}", e)),
            };
        }
        Command::OpenFile(ref path) => {
            return match state.open_file(std::path::Path::new(path)) {
                Ok(()) => AppAction::Continue,
                Err(e) => AppAction::ShowMessage(format!("Error opening file: {:?}", e)),
            };
        }
        Command::Quit => {
            if state.dirty {
                return AppAction::ShowMessage(
                    "Unsaved changes. Use :q! to force quit.".to_string(),
                );
            }
            return AppAction::Quit;
        }
        Command::ForceQuit => return AppAction::ForceQuit,
        Command::Undo => {
            if !state.undo() {
                return AppAction::ShowMessage("Nothing to undo".to_string());
            }
        }
        Command::Redo => {
            if !state.redo() {
                return AppAction::ShowMessage("Nothing to redo".to_string());
            }
        }
        Command::SearchForward(ref query) => {
            if !state.search_forward(query) {
                return AppAction::ShowMessage(format!("Pattern not found: {}", query));
            }
        }
        Command::SearchBackward(ref query) => {
            if !state.search_backward(query) {
                return AppAction::ShowMessage(format!("Pattern not found: {}", query));
            }
        }
        Command::SearchNext => {
            if !state.search_forward(&state.search_query.clone()) {
                return AppAction::ShowMessage(format!("Pattern not found: {}", state.search_query));
            }
        }
        Command::SearchPrevious => {
            if !state.search_backward(&state.search_query.clone()) {
                return AppAction::ShowMessage(format!("Pattern not found: {}", state.search_query));
            }
        }
        _ => {}
    }
    AppAction::Continue
}

use crate::position::Position;

pub struct App {
    #[allow(dead_code)]
    terminal: Terminal,
    state: EditorState,
    renderer: Renderer,
    #[allow(dead_code)]
    registry: CommandRegistry,
    should_quit: bool,
    keymap: Keymap,
    #[allow(dead_code)]
    config: Config,
}

impl App {
    pub fn new(file_path: Option<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let terminal = Terminal::new()?;
        let (cols, rows) = terminal.size()?;
        let mut state = EditorState::new();
        if let Some(path) = file_path {
            if let Err(e) = state.open_file(&path) {
                state.message = Some(format!("Error opening file: {:?}", e));
            }
        }

        let config = Self::load_config();
        state.skk_enabled = config.skk_enabled;

        Ok(Self {
            terminal,
            state,
            renderer: Renderer::new(cols as usize, rows as usize, config.tab_width),
            registry: CommandRegistry::new(),
            should_quit: false,
            keymap: Keymap::new(),
            config,
        })
    }

    fn load_config() -> Config {
        let config_path = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".config/dim/config.toml"))
            .unwrap_or_else(|_| std::path::PathBuf::from("dim.toml"));
        Config::load(&config_path).unwrap_or_default()
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while !self.should_quit {
            self.draw()?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use crossterm::{cursor, terminal, QueueableCommand};
        use std::io::Write;

        self.renderer.ensure_cursor_visible(&self.state);
        let frame = self.renderer.render(&self.state);
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        for (i, row) in frame.rows.iter().enumerate() {
            stdout.queue(cursor::MoveTo(0, i as u16))?;
            stdout.write_all(row.as_bytes())?;
        }

        let (col, row) = self.renderer.cursor_position(&self.state);
        stdout.queue(cursor::MoveTo(col as u16, row as u16))?;
        stdout.queue(cursor::Show)?;
        stdout.flush()?;

        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event = crossterm::event::read()?;
        if let Some(input) = crate::input::parse_crossterm_event(event) {
            match input {
                InputEvent::Resize { cols, rows } => {
                    self.renderer = Renderer::new(cols as usize, rows as usize, self.config.tab_width);
                }
                _ => match self.state.mode {
                    EditorMode::Insert => self.handle_insert_mode(&input),
                    EditorMode::Command => self.handle_command_mode(&input),
                    EditorMode::Search => self.handle_search_mode(&input),
                    EditorMode::Normal => {
                        if let Some(cmd) = self.keymap.handle(&input, EditorMode::Normal) {
                            self.run_command(cmd);
                        }
                    }
                },
            }
        }
        Ok(())
    }

    fn handle_insert_mode(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Key { code: KeyCode::Escape, .. } => {
                self.keymap.clear_prefix();
                self.state.set_mode(EditorMode::Normal);
            }
            InputEvent::Key { code: KeyCode::Char('j'), modifiers } if modifiers.ctrl => {
                self.state.skk_engine.toggle();
            }
            InputEvent::Key { code: KeyCode::Backspace, .. } => {
                if self.state.skk_enabled && self.state.skk_engine.state != crate::skk::SkkState::Direct {
                    let action = self.state.skk_engine.cancel();
                    self.apply_skk_action(action);
                } else if self.state.selection.head.col > 0 || self.state.selection.head.line > 0 {
                    self.state.move_cursor_left();
                    self.state.delete_char();
                }
            }
            InputEvent::Key { code: KeyCode::Delete, .. } => {
                self.state.delete_char();
            }
            InputEvent::Key { code: KeyCode::Enter, .. } => {
                if self.state.skk_enabled && self.state.skk_engine.state != crate::skk::SkkState::Direct {
                    let action = self.state.skk_engine.confirm();
                    self.apply_skk_action(action);
                } else {
                    self.state.insert_at_cursor("\n");
                }
            }
            InputEvent::Key { code: KeyCode::Tab, .. } => {
                self.state.insert_at_cursor("\t");
            }
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                if self.state.skk_enabled && self.state.skk_engine.state != crate::skk::SkkState::Direct {
                    for ch in text.chars() {
                        let action = self.state.skk_engine.process_char(ch);
                        self.apply_skk_action(action);
                    }
                } else {
                    self.state.insert_at_cursor(text);
                }
            }
            _ => {}
        }
    }

    fn apply_skk_action(&mut self, action: SkkAction) {
        match action {
            SkkAction::None => {}
            SkkAction::Insert(text) => {
                self.state.insert_at_cursor(&text);
            }
            SkkAction::Convert { reading, candidate } => {
                // Delete the reading and insert the candidate
                let head = self.state.selection.head;
                let reading_len = reading.chars().count();
                if reading_len > 0 && head.col >= reading_len {
                    let start = Position::new(head.line, head.col - reading_len);
                    self.state.selection = Selection::new(start, head);
                    self.state.delete_selection();
                    self.state.insert_at_cursor(&candidate);
                } else {
                    self.state.insert_at_cursor(&candidate);
                }
            }
            SkkAction::Cancel => {}
        }
    }

    fn handle_command_mode(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Key { code: KeyCode::Escape, .. } => {
                self.keymap.clear_prefix();
                self.state.command_buffer.clear();
                self.state.set_mode(EditorMode::Normal);
            }
            InputEvent::Key { code: KeyCode::Enter, .. } => {
                self.keymap.clear_prefix();
                let cmd_str = self.state.command_buffer.clone();
                self.state.command_buffer.clear();
                self.state.set_mode(EditorMode::Normal);
                self.execute_command_line(&cmd_str);
            }
            InputEvent::Key { code: KeyCode::Backspace, .. } => {
                self.state.command_buffer.pop();
            }
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                self.state.command_buffer.push_str(text);
            }
            _ => {}
        }
    }

    fn handle_search_mode(&mut self, input: &InputEvent) {
        match input {
            InputEvent::Key { code: KeyCode::Escape, .. } => {
                self.keymap.clear_prefix();
                self.state.search_query.clear();
                self.state.set_mode(EditorMode::Normal);
            }
            InputEvent::Key { code: KeyCode::Enter, .. } => {
                self.keymap.clear_prefix();
                let query = self.state.search_query.clone();
                self.state.search_query.clear();
                self.state.set_mode(EditorMode::Normal);
                if !self.state.search_forward(&query) {
                    self.state.message = Some(format!("Pattern not found: {}", query));
                }
            }
            InputEvent::Key { code: KeyCode::Backspace, .. } => {
                self.state.search_query.pop();
            }
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                self.state.search_query.push_str(text);
            }
            _ => {}
        }
    }

    fn execute_command_line(&mut self, cmd_str: &str) {
        let parts: Vec<&str> = cmd_str.splitn(2, ' ').collect();
        let cmd_name = parts[0];
        let arg = parts.get(1).copied();

        match cmd_name {
            "w" | "write" => {
                if let Some(path) = arg {
                    self.run_command(Command::SaveFileAs(path.to_string()));
                } else {
                    self.run_command(Command::SaveFile);
                }
            }
            "q" | "quit" => self.run_command(Command::Quit),
            "q!" | "force-quit" => self.run_command(Command::ForceQuit),
            "wq" => {
                self.run_command(Command::SaveFile);
                self.run_command(Command::Quit);
            }
            "open" => {
                if let Some(path) = arg {
                    self.run_command(Command::OpenFile(path.to_string()));
                } else {
                    self.state.message = Some("open requires a path".to_string());
                }
            }
            "undo" => self.run_command(Command::Undo),
            "redo" => self.run_command(Command::Redo),
            _ => {
                self.state.message = Some(format!("Unknown command: {}", cmd_name));
            }
        }
    }

    fn run_command(&mut self, cmd: Command) {
        let prev_mode = self.state.mode;
        let action = execute_command(cmd, &mut self.state);
        if self.state.mode != prev_mode {
            self.keymap.clear_prefix();
        }
        match action {
            AppAction::Continue => {}
            AppAction::Quit => self.should_quit = true,
            AppAction::ForceQuit => self.should_quit = true,
            AppAction::ShowMessage(msg) => self.state.message = Some(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::LineBuffer;
    use crate::position::Position;
    use crate::selection::Selection;

    #[test]
    fn test_execute_move_left() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab");
        state.selection = Selection::cursor(Position::new(0, 1));
        let action = execute_command(Command::MoveLeft, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.selection.head, Position::new(0, 0));
    }

    #[test]
    fn test_execute_move_right() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab");
        state.selection = Selection::cursor(Position::new(0, 0));
        let action = execute_command(Command::MoveRight, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.selection.head, Position::new(0, 1));
    }

    #[test]
    fn test_execute_enter_insert_mode() {
        let mut state = EditorState::new();
        let action = execute_command(Command::EnterInsertMode, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_execute_enter_normal_mode() {
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Insert);
        let action = execute_command(Command::EnterNormalMode, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn test_execute_delete_selection() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        let action = execute_command(Command::DeleteSelection, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.buffer.to_string(), "hello ");
    }

    #[test]
    fn test_execute_undo() {
        let mut state = EditorState::new();
        state.insert_at_cursor("hello");
        let action = execute_command(Command::Undo, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.buffer.to_string(), "");
    }

    #[test]
    fn test_execute_undo_nothing() {
        let mut state = EditorState::new();
        let action = execute_command(Command::Undo, &mut state);
        assert_eq!(action, AppAction::ShowMessage("Nothing to undo".to_string()));
    }

    #[test]
    fn test_execute_redo() {
        let mut state = EditorState::new();
        state.insert_at_cursor("hello");
        state.undo();
        let action = execute_command(Command::Redo, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.buffer.to_string(), "hello");
    }

    #[test]
    fn test_execute_quit_clean() {
        let mut state = EditorState::new();
        let action = execute_command(Command::Quit, &mut state);
        assert_eq!(action, AppAction::Quit);
    }

    #[test]
    fn test_execute_quit_dirty() {
        let mut state = EditorState::new();
        state.dirty = true;
        let action = execute_command(Command::Quit, &mut state);
        assert_eq!(
            action,
            AppAction::ShowMessage("Unsaved changes. Use :q! to force quit.".to_string())
        );
    }

    #[test]
    fn test_execute_force_quit() {
        let mut state = EditorState::new();
        state.dirty = true;
        let action = execute_command(Command::ForceQuit, &mut state);
        assert_eq!(action, AppAction::ForceQuit);
    }

    #[test]
    fn test_execute_paste_after() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab");
        state.selection = Selection::cursor(Position::new(0, 0));
        state.yank_buffer = "XY".to_string();
        let action = execute_command(Command::PasteAfter, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.buffer.to_string(), "aXYb");
    }

    #[test]
    fn test_execute_paste_before() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab");
        state.selection = Selection::cursor(Position::new(0, 1));
        state.yank_buffer = "XY".to_string();
        let action = execute_command(Command::PasteBefore, &mut state);
        assert_eq!(action, AppAction::Continue);
        assert_eq!(state.buffer.to_string(), "aXYb");
    }
}
