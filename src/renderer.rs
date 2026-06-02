use crate::editor_state::{EditorMode, EditorState};

pub struct Frame {
    pub rows: Vec<String>,
}

pub struct Renderer {
    width: usize,
    height: usize,
    scroll_offset: usize,
    tab_width: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize, tab_width: usize) -> Self {
        Self {
            width,
            height,
            scroll_offset: 0,
            tab_width,
        }
    }

    pub fn render(&self, state: &EditorState) -> Frame {
        let mut rows = Vec::with_capacity(self.height);
        let text_height = self.height.saturating_sub(1);

        for i in 0..text_height {
            let line_idx = self.scroll_offset + i;
            if let Some(line) = state.buffer.line(line_idx) {
                let rendered = truncate_to_width(line, self.width, self.tab_width);
                rows.push(rendered);
            } else {
                rows.push(String::new());
            }
        }

        let status_row = match state.mode {
            EditorMode::Command => format!(":{}", state.command_buffer),
            EditorMode::Search => format!("/{}", state.search_query),
            _ => self.build_status_line(state),
        };
        rows.push(status_row);

        Frame { rows }
    }

    /// Adjusts scroll offset so the cursor is visible on screen.
    pub fn ensure_cursor_visible(&mut self, state: &EditorState) {
        let text_height = self.height.saturating_sub(1);
        let cursor_line = state.selection.head.line;

        if cursor_line < self.scroll_offset {
            self.scroll_offset = cursor_line;
        } else if cursor_line >= self.scroll_offset + text_height {
            self.scroll_offset = cursor_line.saturating_sub(text_height - 1);
        }
    }

    /// Returns (col, row) in terminal coordinates for the current selection head.
    pub fn cursor_position(&self, state: &EditorState) -> (usize, usize) {
        let row = state.selection.head.line.saturating_sub(self.scroll_offset);
        let row = row.min(self.height.saturating_sub(2));

        let mut col = 0;
        if let Some(line) = state.buffer.line(state.selection.head.line) {
            let target_col = state.selection.head.col;
            let mut char_idx = 0;
            for ch in line.chars() {
                if char_idx >= target_col {
                    break;
                }
                let cw = if ch == '\t' {
                    let next_stop = (col / self.tab_width + 1) * self.tab_width;
                    next_stop - col
                } else {
                    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
                };
                col += cw;
                char_idx += 1;
            }
        } else {
            col = state.selection.head.col;
        }
        let col = col.min(self.width);
        (col, row)
    }

    fn build_status_line(&self, state: &EditorState) -> String {
        if let Some(ref msg) = state.message {
            return truncate_to_width(msg, self.width, self.tab_width);
        }

        let mut mode_str = match state.mode {
            EditorMode::Normal => "NORMAL".to_string(),
            EditorMode::Insert => "INSERT".to_string(),
            EditorMode::Command => "COMMAND".to_string(),
            EditorMode::Search => "SEARCH".to_string(),
        };

        if state.mode == EditorMode::Insert && state.skk_enabled {
            let skk_label = match state.skk_engine.state {
                crate::skk::SkkState::Direct => "",
                crate::skk::SkkState::Hiragana => "[あ]",
                crate::skk::SkkState::Katakana => "[ア]",
                crate::skk::SkkState::Converting => "[変換]",
                crate::skk::SkkState::Registering => "[登録]",
            };
            if !skk_label.is_empty() {
                mode_str = format!("{} {}", mode_str, skk_label);
            }
        }

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
            truncate_to_width(&left, self.width, self.tab_width)
        } else {
            let pad = self.width - total_w;
            format!("{}{}{}", left, " ".repeat(pad), right)
        }
    }
}

fn truncate_to_width(s: &str, max_width: usize, tab_width: usize) -> String {
    let mut result = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = if ch == '\t' {
            let next_stop = (w / tab_width + 1) * tab_width;
            next_stop - w
        } else {
            unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
        };
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
        let r = Renderer::new(80, 24, 4);
        let frame = r.render(&EditorState::new());
        assert_eq!(frame.rows.len(), 24);
    }

    #[test]
    fn test_render_empty_buffer() {
        let r = Renderer::new(80, 3, 4);
        let state = EditorState::new();
        let frame = r.render(&state);
        assert_eq!(frame.rows.len(), 3);
        assert_eq!(frame.rows[0], "");
        assert_eq!(frame.rows[1], "");
        assert!(frame.rows[2].contains("NORMAL"));
    }

    #[test]
    fn test_render_text_lines() {
        let r = Renderer::new(80, 4, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello\nworld\n!");
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "hello");
        assert_eq!(frame.rows[1], "world");
        assert_eq!(frame.rows[2], "!");
    }

    #[test]
    fn test_render_truncates_long_lines() {
        let r = Renderer::new(5, 3, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("abcdefgh");
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "abcde");
    }

    #[test]
    fn test_render_status_line_no_name() {
        let r = Renderer::new(40, 2, 4);
        let state = EditorState::new();
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("NORMAL"));
        assert!(status.contains("[No Name]"));
        assert!(status.contains("1:1"));
    }

    #[test]
    fn test_render_status_line_with_file_and_dirty() {
        let r = Renderer::new(50, 2, 4);
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
        let r = Renderer::new(40, 2, 4);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Command);
        state.command_buffer = "write".to_string();
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], ":write");
    }

    #[test]
    fn test_cursor_position() {
        let r = Renderer::new(80, 24, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("hello\nworld");
        state.selection = Selection::cursor(Position::new(1, 3));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 3);
        assert_eq!(row, 1);
    }

    #[test]
    fn test_cursor_position_clamped() {
        let r = Renderer::new(5, 2, 4);
        let mut state = EditorState::new();
        state.selection = Selection::cursor(Position::new(100, 100));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 5);
        assert_eq!(row, 0);
    }

    #[test]
    fn test_render_japanese_truncated() {
        let r = Renderer::new(5, 2, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("日本語");
        let frame = r.render(&state);
        // 日本 = 4 width, 日 = 2 width, so at width 5 we can only fit 日+本 (4) but not 日+本+語 (6).
        // So it should be "日本" (width 4) since 5 >= 4 but 5 < 6.
        assert_eq!(frame.rows[0], "日本");
    }

    #[test]
    fn test_render_status_line_shows_message() {
        let r = Renderer::new(40, 2, 4);
        let mut state = EditorState::new();
        state.message = Some("File saved".to_string());
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], "File saved");
    }

    #[test]
    fn test_render_search_mode() {
        let r = Renderer::new(40, 2, 4);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Search);
        state.search_query = "hello".to_string();
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], "/hello");
    }

    #[test]
    fn test_scroll_offset_renders_correct_lines() {
        let mut r = Renderer::new(10, 3, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("line0\nline1\nline2\nline3\nline4");
        r.scroll_offset = 2;
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "line2");
        assert_eq!(frame.rows[1], "line3");
    }

    #[test]
    fn test_ensure_cursor_visible_scrolls_down() {
        let mut r = Renderer::new(10, 3, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("line0\nline1\nline2\nline3\nline4");
        state.selection = Selection::cursor(Position::new(3, 0));
        r.ensure_cursor_visible(&state);
        assert_eq!(r.scroll_offset, 2);
    }

    #[test]
    fn test_ensure_cursor_visible_scrolls_up() {
        let mut r = Renderer::new(10, 3, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("line0\nline1\nline2\nline3\nline4");
        r.scroll_offset = 3;
        state.selection = Selection::cursor(Position::new(1, 0));
        r.ensure_cursor_visible(&state);
        assert_eq!(r.scroll_offset, 1);
    }

    #[test]
    fn test_cursor_position_with_scroll() {
        let mut r = Renderer::new(10, 5, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("line0\nline1\nline2\nline3\nline4");
        r.scroll_offset = 2;
        state.selection = Selection::cursor(Position::new(3, 2));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 2);
        assert_eq!(row, 1);
    }

    #[test]
    fn test_cursor_position_japanese() {
        let r = Renderer::new(80, 24, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("日本語");
        // Cursor after "日" (1 char)
        state.selection = Selection::cursor(Position::new(0, 1));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 2); // full-width = 2 columns
        assert_eq!(row, 0);

        // Cursor after "日本" (2 chars)
        state.selection = Selection::cursor(Position::new(0, 2));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 4);
        assert_eq!(row, 0);
    }

    #[test]
    fn test_cursor_position_with_tab() {
        let r = Renderer::new(80, 24, 4);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from_str("a\tb\tc");
        // Cursor after "a\t" (2 chars)
        state.selection = Selection::cursor(Position::new(0, 2));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 4); // 'a' (1) + tab to next stop (4) = 4
        assert_eq!(row, 0);

        // Cursor after "a\tb\t" (4 chars)
        state.selection = Selection::cursor(Position::new(0, 4));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 8); // 4 + 'b'(1) + tab(3) = 8
        assert_eq!(row, 0);
    }
}
