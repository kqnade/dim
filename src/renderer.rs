use crate::editor_state::{EditorMode, EditorState};

pub struct Frame {
    pub rows: Vec<String>,
}

pub struct Renderer {
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn render(&self, state: &EditorState) -> Frame {
        let mut rows = Vec::with_capacity(self.height);
        let text_height = self.height.saturating_sub(1);

        for i in 0..text_height {
            if let Some(line) = state.buffer.line(i) {
                let rendered = truncate_to_width(line, self.width);
                rows.push(rendered);
            } else {
                rows.push(String::new());
            }
        }

        let status_row = match state.mode {
            EditorMode::Command => format!(":{}", state.command_buffer),
            _ => self.build_status_line(state),
        };
        rows.push(status_row);

        Frame { rows }
    }

    /// Returns (col, row) in terminal coordinates for the current selection head.
    /// Scroll offset is not yet implemented; assumes file fits on screen.
    pub fn cursor_position(&self, state: &EditorState) -> (usize, usize) {
        let row = state.selection.head.line.min(self.height.saturating_sub(2));
        let col = state.selection.head.col.min(self.width);
        (col, row)
    }

    fn build_status_line(&self, state: &EditorState) -> String {
        if let Some(ref msg) = state.message {
            return truncate_to_width(msg, self.width);
        }

        let mode_str = match state.mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
            EditorMode::Search => "SEARCH",
        };

        let file_name = state
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]");

        let dirty_mark = if state.dirty { " [+]" } else { "" };
        let pos = format!("{}:{}", state.selection.head.line + 1, state.selection.head.col + 1);

        let left = format!("-- {} -- {}{}", mode_str, file_name, dirty_mark);
        let right = pos;

        let left_w = unicode_width::UnicodeWidthStr::width(left.as_str());
        let right_w = unicode_width::UnicodeWidthStr::width(right.as_str());
        let sep_w = 1;
        let total_w = left_w + sep_w + right_w;

        if total_w >= self.width {
            truncate_to_width(&left, self.width)
        } else {
            let pad = self.width - total_w;
            format!("{}{}{}", left, " ".repeat(pad), right)
        }
    }
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_width {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::LineBuffer;
    use crate::position::Position;
    use crate::selection::Selection;
    use std::path::PathBuf;

    #[test]
    fn test_renderer_new() {
        let r = Renderer::new(80, 24);
        let frame = r.render(&EditorState::new());
        assert_eq!(frame.rows.len(), 24);
    }

    #[test]
    fn test_render_empty_buffer() {
        let r = Renderer::new(80, 3);
        let state = EditorState::new();
        let frame = r.render(&state);
        assert_eq!(frame.rows.len(), 3);
        assert_eq!(frame.rows[0], "");
        assert_eq!(frame.rows[1], "");
        assert!(frame.rows[2].contains("NORMAL"));
    }

    #[test]
    fn test_render_text_lines() {
        let r = Renderer::new(80, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello\nworld\n!");
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "hello");
        assert_eq!(frame.rows[1], "world");
        assert_eq!(frame.rows[2], "!");
    }

    #[test]
    fn test_render_truncates_long_lines() {
        let r = Renderer::new(5, 3);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("abcdefgh");
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "abcde");
    }

    #[test]
    fn test_render_status_line_no_name() {
        let r = Renderer::new(40, 2);
        let state = EditorState::new();
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("NORMAL"));
        assert!(status.contains("[No Name]"));
        assert!(status.contains("1:1"));
    }

    #[test]
    fn test_render_status_line_with_file_and_dirty() {
        let r = Renderer::new(50, 2);
        let mut state = EditorState::new();
        state.file_path = Some(PathBuf::from("/tmp/test.txt"));
        state.dirty = true;
        state.selection = Selection::cursor(Position::new(2, 5));
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("test.txt"));
        assert!(status.contains("[+]"));
        assert!(status.contains("3:6"));
    }

    #[test]
    fn test_render_command_mode() {
        let r = Renderer::new(40, 2);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Command);
        state.command_buffer = "write".to_string();
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], ":write");
    }

    #[test]
    fn test_cursor_position() {
        let r = Renderer::new(80, 24);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello\nworld");
        state.selection = Selection::cursor(Position::new(1, 3));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 3);
        assert_eq!(row, 1);
    }

    #[test]
    fn test_cursor_position_clamped() {
        let r = Renderer::new(5, 2);
        let mut state = EditorState::new();
        state.selection = Selection::cursor(Position::new(100, 100));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 5);
        assert_eq!(row, 0);
    }

    #[test]
    fn test_render_japanese_truncated() {
        let r = Renderer::new(5, 2);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("日本語");
        let frame = r.render(&state);
        // 日本 = 4 width, 日 = 2 width, so at width 5 we can only fit 日+本 (4) but not 日+本+語 (6).
        // So it should be "日本" (width 4) since 5 >= 4 but 5 < 6.
        assert_eq!(frame.rows[0], "日本");
    }

    #[test]
    fn test_render_status_line_shows_message() {
        let r = Renderer::new(40, 2);
        let mut state = EditorState::new();
        state.message = Some("File saved".to_string());
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], "File saved");
    }
}
