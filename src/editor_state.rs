use crate::buffer::LineBuffer;
use crate::file_io::{read_file, write_file, FileError};
use crate::position::Position;
use crate::selection::Selection;
use crate::skk::SkkEngine;
use crate::undo::{EditOp, Transaction, UndoManager};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
    Search,
}

pub struct EditorState {
    pub buffer: LineBuffer,
    pub selection: Selection,
    pub mode: EditorMode,
    pub message: Option<String>,
    pub dirty: bool,
    pub file_path: Option<PathBuf>,
    pub command_buffer: String,
    pub yank_buffer: String,
    pub search_query: String,
    pub skk_engine: SkkEngine,
    pub skk_enabled: bool,
    undo_manager: UndoManager,
    #[allow(dead_code)]
    current_transaction: Transaction,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffer: LineBuffer::new(),
            selection: Selection::cursor(Position::new(0, 0)),
            mode: EditorMode::Normal,
            message: None,
            dirty: false,
            file_path: None,
            command_buffer: String::new(),
            yank_buffer: String::new(),
            search_query: String::new(),
            skk_engine: SkkEngine::new(),
            skk_enabled: false,
            undo_manager: UndoManager::new(),
            current_transaction: Transaction::new(),
        }
    }

    pub fn open_file(&mut self, path: &std::path::Path) -> Result<(), FileError> {
        let text = read_file(path)?;
        self.buffer = LineBuffer::from_str(&text);
        self.selection = Selection::cursor(Position::new(0, 0));
        self.file_path = Some(path.to_path_buf());
        self.dirty = false;
        self.message = None;
        self.command_buffer.clear();
        self.search_query.clear();
        self.undo_manager = UndoManager::new();
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), FileError> {
        if let Some(ref path) = self.file_path {
            write_file(path, &self.buffer.to_string())?;
            self.dirty = false;
            Ok(())
        } else {
            Err(FileError::WriteFailed)
        }
    }

    pub fn save_file_as(&mut self, path: &std::path::Path) -> Result<(), FileError> {
        write_file(path, &self.buffer.to_string())?;
        self.file_path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    pub fn insert_at_cursor(&mut self, text: &str) -> Position {
        let mut txn = Transaction::new();

        if !self.selection.is_empty() {
            let (start, end) = self.selection.sorted();
            let deleted = self.buffer.delete_range(start, end);
            txn.push(EditOp::Delete {
                pos: start,
                text: deleted,
            });
            self.selection = Selection::cursor(start);
        }

        let pos = self.selection.head;
        let end = self.buffer.insert(pos, text);
        txn.push(EditOp::Insert {
            pos,
            text: text.to_string(),
        });

        self.selection = Selection::cursor(end);
        self.dirty = true;
        self.push_transaction(txn);
        end
    }

    pub fn yank_selection(&mut self) -> String {
        let (start, end) = self.selection.sorted();
        if start == end {
            return String::new();
        }
        let yanked = self.buffer.delete_range(start, end);
        self.buffer.insert(start, &yanked);
        self.yank_buffer = yanked.clone();
        self.selection = Selection::new(start, end);
        yanked
    }

    pub fn delete_selection(&mut self) -> String {
        let (start, end) = self.selection.sorted();
        let deleted = self.buffer.delete_range(start, end);
        self.selection = Selection::cursor(start);
        self.yank_buffer = deleted.clone();

        if !deleted.is_empty() {
            let txn = Transaction::with_ops(vec![EditOp::Delete {
                pos: start,
                text: deleted.clone(),
            }]);
            self.dirty = true;
            self.push_transaction(txn);
        }
        deleted
    }

    /// Deletes the character under the cursor (for empty selection).
    /// If at end of line and not last line, joins with next line.
    pub fn delete_char(&mut self) {
        let pos = self.selection.head;
        let line_len = self.buffer.line_len(pos.line).unwrap_or(0);
        if pos.col < line_len {
            self.selection = Selection::new(pos, Position::new(pos.line, pos.col + 1));
            self.delete_selection();
        } else if pos.line + 1 < self.buffer.line_count() {
            self.selection = Selection::new(pos, Position::new(pos.line + 1, 0));
            self.delete_selection();
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(txn) = self.undo_manager.undo() {
            self.buffer.apply_transaction_inverse(&txn);
            self.selection = Selection::cursor(self.clamp_position(self.selection.head));
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(txn) = self.undo_manager.redo() {
            self.buffer.apply_transaction(&txn);
            self.selection = Selection::cursor(self.clamp_position(self.selection.head));
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_manager.can_redo()
    }

    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    pub fn move_head_to(&mut self, pos: Position) {
        let clamped = self.clamp_position(pos);
        self.selection = Selection::cursor(clamped);
    }

    pub fn move_cursor_left(&mut self) {
        let head = self.selection.head;
        if head.col > 0 {
            self.selection = Selection::cursor(Position::new(head.line, head.col - 1));
        } else if head.line > 0 {
            let prev_line = head.line - 1;
            let prev_len = self.buffer.line_len(prev_line).unwrap_or(0);
            self.selection = Selection::cursor(Position::new(prev_line, prev_len));
        }
    }

    pub fn move_cursor_right(&mut self) {
        let head = self.selection.head;
        let line_len = self.buffer.line_len(head.line).unwrap_or(0);
        if head.col < line_len {
            self.selection = Selection::cursor(Position::new(head.line, head.col + 1));
        } else if head.line + 1 < self.buffer.line_count() {
            self.selection = Selection::cursor(Position::new(head.line + 1, 0));
        }
    }

    pub fn move_cursor_up(&mut self) {
        let head = self.selection.head;
        if head.line > 0 {
            let new_line = head.line - 1;
            let new_len = self.buffer.line_len(new_line).unwrap_or(0);
            let new_col = head.col.min(new_len);
            self.selection = Selection::cursor(Position::new(new_line, new_col));
        }
    }

    pub fn move_cursor_down(&mut self) {
        let head = self.selection.head;
        if head.line + 1 < self.buffer.line_count() {
            let new_line = head.line + 1;
            let new_len = self.buffer.line_len(new_line).unwrap_or(0);
            let new_col = head.col.min(new_len);
            self.selection = Selection::cursor(Position::new(new_line, new_col));
        }
    }

    pub fn search_forward(&mut self, query: &str) -> bool {
        if query.is_empty() {
            return false;
        }
        let start_line = self.selection.head.line;
        let start_col = self.selection.head.col;

        // Search current line after cursor
        if let Some(line) = self.buffer.line(start_line) {
            if let Some(idx) = line[start_col..].find(query) {
                let found_col = start_col + idx;
                self.selection = Selection::cursor(Position::new(start_line, found_col));
                return true;
            }
        }

        // Search subsequent lines
        for line_idx in (start_line + 1)..self.buffer.line_count() {
            if let Some(line) = self.buffer.line(line_idx) {
                if let Some(idx) = line.find(query) {
                    self.selection = Selection::cursor(Position::new(line_idx, idx));
                    return true;
                }
            }
        }

        false
    }

    pub fn search_backward(&mut self, query: &str) -> bool {
        if query.is_empty() {
            return false;
        }
        let start_line = self.selection.head.line;
        let start_col = self.selection.head.col;

        // Search current line before cursor
        if let Some(line) = self.buffer.line(start_line) {
            let before = &line[..start_col];
            if let Some(idx) = before.rfind(query) {
                self.selection = Selection::cursor(Position::new(start_line, idx));
                return true;
            }
        }

        // Search preceding lines
        for line_idx in (0..start_line).rev() {
            if let Some(line) = self.buffer.line(line_idx) {
                if let Some(idx) = line.rfind(query) {
                    self.selection = Selection::cursor(Position::new(line_idx, idx));
                    return true;
                }
            }
        }

        false
    }

    pub fn clamp_position(&self, pos: Position) -> Position {
        let line = pos.line.min(self.buffer.line_count().saturating_sub(1));
        let col = pos.col.min(self.buffer.line_len(line).unwrap_or(0));
        Position::new(line, col)
    }

    fn push_transaction(&mut self, txn: Transaction) {
        if !txn.is_empty() {
            self.undo_manager.push(txn);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn test_editor_state_new() {
        let state = EditorState::new();
        assert_eq!(state.buffer.to_string(), "");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 0)));
        assert_eq!(state.mode, EditorMode::Normal);
        assert!(!state.dirty);
        assert_eq!(state.file_path, None);
        assert!(!state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_open_file() {
        let path = temp_dir().join("dim_test_open.txt");
        fs::write(&path, "hello\nworld").unwrap();
        let mut state = EditorState::new();
        state.open_file(&path).unwrap();
        assert_eq!(state.buffer.to_string(), "hello\nworld");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 0)));
        assert!(!state.dirty);
        assert_eq!(state.file_path, Some(path.clone()));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_open_file_not_found() {
        let mut state = EditorState::new();
        let path = temp_dir().join("dim_test_open_nonexistent.txt");
        let result = state.open_file(&path);
        assert_eq!(result, Err(FileError::NotFound));
    }

    #[test]
    fn test_save_file() {
        let path = temp_dir().join("dim_test_save.txt");
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("save me");
        state.file_path = Some(path.clone());
        state.dirty = true;
        state.save_file().unwrap();
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "save me");
        assert!(!state.dirty);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_save_file_no_path() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("no path");
        let result = state.save_file();
        assert_eq!(result, Err(FileError::WriteFailed));
    }

    #[test]
    fn test_save_file_as() {
        let path = temp_dir().join("dim_test_save_as.txt");
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("new content");
        state.dirty = true;
        state.save_file_as(&path).unwrap();
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "new content");
        assert!(!state.dirty);
        assert_eq!(state.file_path, Some(path.clone()));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_insert_at_cursor() {
        let mut state = EditorState::new();
        let end = state.insert_at_cursor("abc");
        assert_eq!(end, Position::new(0, 3));
        assert_eq!(state.buffer.to_string(), "abc");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 3)));
        assert!(state.dirty);
        assert!(state.can_undo());
    }

    #[test]
    fn test_insert_replaces_selection() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        state.insert_at_cursor("dim");
        assert_eq!(state.buffer.to_string(), "hello dim");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 9)));
        assert!(state.can_undo());
    }

    #[test]
    fn test_delete_selection() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        let deleted = state.delete_selection();
        assert_eq!(deleted, "world");
        assert_eq!(state.buffer.to_string(), "hello ");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 6)));
        assert!(state.dirty);
        assert!(state.can_undo());
    }

    #[test]
    fn test_delete_empty_selection() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("abc");
        state.selection = Selection::cursor(Position::new(0, 1));
        let deleted = state.delete_selection();
        assert_eq!(deleted, "");
        assert_eq!(state.buffer.to_string(), "abc");
        assert!(!state.can_undo());
    }

    #[test]
    fn test_delete_char() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("abc");
        state.selection = Selection::cursor(Position::new(0, 1));
        state.delete_char();
        assert_eq!(state.buffer.to_string(), "ac");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 1)));
        assert!(state.can_undo());
    }

    #[test]
    fn test_delete_char_joins_lines() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab\ncd");
        state.selection = Selection::cursor(Position::new(0, 2));
        state.delete_char();
        assert_eq!(state.buffer.to_string(), "abcd");
        assert_eq!(state.selection, Selection::cursor(Position::new(0, 2)));
    }

    #[test]
    fn test_undo_insert() {
        let mut state = EditorState::new();
        state.insert_at_cursor("hello");
        assert_eq!(state.buffer.to_string(), "hello");
        let undone = state.undo();
        assert!(undone);
        assert_eq!(state.buffer.to_string(), "");
        assert!(!state.can_undo());
        assert!(state.can_redo());
    }

    #[test]
    fn test_redo_insert() {
        let mut state = EditorState::new();
        state.insert_at_cursor("hello");
        state.undo();
        let redone = state.redo();
        assert!(redone);
        assert_eq!(state.buffer.to_string(), "hello");
        assert!(state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_undo_delete() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::new(Position::new(0, 5), Position::new(0, 11));
        state.delete_selection();
        assert_eq!(state.buffer.to_string(), "hello");
        state.undo();
        assert_eq!(state.buffer.to_string(), "hello world");
    }

    #[test]
    fn test_undo_redo_multiple() {
        let mut state = EditorState::new();
        state.insert_at_cursor("a");
        state.insert_at_cursor("b");
        state.insert_at_cursor("c");
        assert_eq!(state.buffer.to_string(), "abc");
        state.undo();
        assert_eq!(state.buffer.to_string(), "ab");
        state.undo();
        assert_eq!(state.buffer.to_string(), "a");
        state.redo();
        assert_eq!(state.buffer.to_string(), "ab");
        state.redo();
        assert_eq!(state.buffer.to_string(), "abc");
    }

    #[test]
    fn test_move_cursor_left() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab\ncd");
        state.selection = Selection::cursor(Position::new(1, 1));
        state.move_cursor_left();
        assert_eq!(state.selection.head, Position::new(1, 0));
        state.move_cursor_left();
        assert_eq!(state.selection.head, Position::new(0, 2));
        state.move_cursor_left();
        assert_eq!(state.selection.head, Position::new(0, 1));
    }

    #[test]
    fn test_move_cursor_left_at_origin() {
        let mut state = EditorState::new();
        state.selection = Selection::cursor(Position::new(0, 0));
        state.move_cursor_left();
        assert_eq!(state.selection.head, Position::new(0, 0));
    }

    #[test]
    fn test_move_cursor_right() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab\ncd");
        state.selection = Selection::cursor(Position::new(0, 1));
        state.move_cursor_right();
        assert_eq!(state.selection.head, Position::new(0, 2));
        state.move_cursor_right();
        assert_eq!(state.selection.head, Position::new(1, 0));
        state.move_cursor_right();
        assert_eq!(state.selection.head, Position::new(1, 1));
    }

    #[test]
    fn test_move_cursor_right_at_end() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("a");
        state.selection = Selection::cursor(Position::new(0, 1));
        state.move_cursor_right();
        assert_eq!(state.selection.head, Position::new(0, 1));
    }

    #[test]
    fn test_move_cursor_up() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello\nworld");
        state.selection = Selection::cursor(Position::new(1, 3));
        state.move_cursor_up();
        assert_eq!(state.selection.head, Position::new(0, 3));
    }

    #[test]
    fn test_move_cursor_up_clamps_col() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hi\nworld");
        state.selection = Selection::cursor(Position::new(1, 5));
        state.move_cursor_up();
        assert_eq!(state.selection.head, Position::new(0, 2));
    }

    #[test]
    fn test_move_cursor_up_at_top() {
        let mut state = EditorState::new();
        state.selection = Selection::cursor(Position::new(0, 0));
        state.move_cursor_up();
        assert_eq!(state.selection.head, Position::new(0, 0));
    }

    #[test]
    fn test_move_cursor_down() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello\nworld");
        state.selection = Selection::cursor(Position::new(0, 3));
        state.move_cursor_down();
        assert_eq!(state.selection.head, Position::new(1, 3));
    }

    #[test]
    fn test_move_cursor_down_clamps_col() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("world\nhi");
        state.selection = Selection::cursor(Position::new(0, 5));
        state.move_cursor_down();
        assert_eq!(state.selection.head, Position::new(1, 2));
    }

    #[test]
    fn test_move_cursor_down_at_bottom() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello");
        state.selection = Selection::cursor(Position::new(0, 0));
        state.move_cursor_down();
        assert_eq!(state.selection.head, Position::new(0, 0));
    }

    #[test]
    fn test_clamp_position() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("ab\ncde");
        assert_eq!(state.clamp_position(Position::new(0, 100)), Position::new(0, 2));
        assert_eq!(state.clamp_position(Position::new(100, 0)), Position::new(1, 0));
        assert_eq!(state.clamp_position(Position::new(1, 100)), Position::new(1, 3));
    }

    #[test]
    fn test_set_mode() {
        let mut state = EditorState::new();
        assert_eq!(state.mode, EditorMode::Normal);
        state.set_mode(EditorMode::Insert);
        assert_eq!(state.mode, EditorMode::Insert);
    }

    #[test]
    fn test_new_buffers_empty() {
        let state = EditorState::new();
        assert_eq!(state.command_buffer, "");
        assert_eq!(state.yank_buffer, "");
    }

    #[test]
    fn test_yank_selection() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        let yanked = state.yank_selection();
        assert_eq!(yanked, "world");
        assert_eq!(state.yank_buffer, "world");
        assert_eq!(state.buffer.to_string(), "hello world");
        assert_eq!(state.selection, Selection::new(Position::new(0, 6), Position::new(0, 11)));
    }

    #[test]
    fn test_delete_sets_yank_buffer() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        state.delete_selection();
        assert_eq!(state.yank_buffer, "world");
    }

    #[test]
    fn test_search_forward() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world\nhello dim");
        state.selection = Selection::cursor(Position::new(0, 0));
        let found = state.search_forward("hello");
        assert!(found);
        assert_eq!(state.selection.head, Position::new(0, 0));
    }

    #[test]
    fn test_search_forward_next_line() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world\nhello dim");
        state.selection = Selection::cursor(Position::new(0, 2));
        let found = state.search_forward("hello");
        assert!(found);
        assert_eq!(state.selection.head, Position::new(1, 0));
    }

    #[test]
    fn test_search_forward_not_found() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world");
        state.selection = Selection::cursor(Position::new(0, 0));
        let found = state.search_forward("xyz");
        assert!(!found);
    }

    #[test]
    fn test_search_backward() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world\nhello dim");
        state.selection = Selection::cursor(Position::new(1, 5));
        let found = state.search_backward("hello");
        assert!(found);
        assert_eq!(state.selection.head, Position::new(1, 0));
    }

    #[test]
    fn test_search_backward_prev_line() {
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello world\nhello dim");
        state.selection = Selection::cursor(Position::new(1, 0));
        let found = state.search_backward("hello");
        assert!(found);
        assert_eq!(state.selection.head, Position::new(0, 0));
    }
}
