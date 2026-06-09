use crate::editor_state::{EditorMode, EditorState};
use crate::selection::Selection;

pub struct Frame {
    pub rows: Vec<String>,
}

pub struct Renderer {
    width: usize,
    height: usize,
    scroll_offset: usize,
    tab_width: usize,
    show_line_numbers: bool,
    show_relative_line_numbers: bool,
}

impl Renderer {
    pub fn new(
        width: usize,
        height: usize,
        tab_width: usize,
        show_line_numbers: bool,
        show_relative_line_numbers: bool,
    ) -> Self {
        Self {
            width,
            height,
            scroll_offset: 0,
            tab_width,
            show_line_numbers,
            show_relative_line_numbers,
        }
    }

    pub fn render(&self, state: &EditorState) -> Frame {
        let mut rows = Vec::with_capacity(self.height);
        let text_height = self.height.saturating_sub(1);
        let gutter_width = if self.show_line_numbers {
            self.compute_gutter_width(state)
        } else {
            0
        };
        let text_width = self.width.saturating_sub(gutter_width);

        for i in 0..text_height {
            let line_idx = self.scroll_offset + i;
            let mut row = String::new();

            if self.show_line_numbers {
                let num = if self.show_relative_line_numbers && line_idx != state.selection.head.line
                {
                    let rel = line_idx.abs_diff(state.selection.head.line);
                    format!("{:>width$} ", rel, width = gutter_width.saturating_sub(1))
                } else {
                    let display_line = line_idx + 1;
                    format!("{:>width$} ", display_line, width = gutter_width.saturating_sub(1))
                };
                row.push_str(&num);
            }

            if let Some(line) = state.buffer.line(line_idx) {
                let line_len = line.chars().count();
                let rendered = if let Some((sel_start, sel_end)) =
                    selection_range_on_line(&state.selection, line_idx, line_len)
                {
                    let highlighted = highlight_line(line, sel_start, sel_end);
                    truncate_ansi_to_width(&highlighted, text_width, self.tab_width)
                } else {
                    truncate_to_width(line, text_width, self.tab_width)
                };
                row.push_str(&rendered);
            }

            rows.push(row);
        }

        let status_row = match state.mode {
            EditorMode::Command => format!(":{}", state.command_buffer),
            EditorMode::Search => format!("/{}", state.search_query),
            _ => self.build_status_line(state),
        };
        rows.push(status_row);

        Frame { rows }
    }

    fn compute_gutter_width(&self, state: &EditorState) -> usize {
        let max_line = state.buffer.line_count().max(1);
        let digits = max_line.to_string().len();
        digits + 1 // digits + space separator
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

        let gutter_width = if self.show_line_numbers {
            self.compute_gutter_width(state)
        } else {
            0
        };

        let mut col = gutter_width;
        if let Some(line) = state.buffer.line(state.selection.head.line) {
            let target_col = state.selection.head.col;
            for (char_idx, ch) in line.chars().enumerate() {
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
            }
        } else {
            col = state.selection.head.col + gutter_width;
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

fn selection_range_on_line(selection: &Selection, line_idx: usize, line_len: usize) -> Option<(usize, usize)> {
    if selection.is_empty() {
        return None;
    }
    let (start, end) = selection.sorted();
    if start.line > line_idx || end.line < line_idx {
        return None;
    }
    let sel_start = if start.line == line_idx { start.col } else { 0 };
    let sel_end = if end.line == line_idx { end.col } else { line_len };
    if sel_start >= sel_end {
        return None;
    }
    Some((sel_start, sel_end))
}

fn highlight_line(line: &str, sel_start: usize, sel_end: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let before: String = chars[..sel_start.min(chars.len())].iter().collect();
    let selected: String = chars[sel_start.min(chars.len())..sel_end.min(chars.len())]
        .iter()
        .collect();
    let after: String = chars[sel_end.min(chars.len())..].iter().collect();
    format!("{}\x1b[7m{}\x1b[0m{}", before, selected, after)
}

fn truncate_ansi_to_width(s: &str, max_width: usize, tab_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            result.push(ch);
            while let Some(&next) = chars.peek() {
                result.push(next);
                chars.next();
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }

        let cw = if ch == '\t' {
            let next_stop = (width / tab_width + 1) * tab_width;
            next_stop - width
        } else {
            unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
        };

        if width + cw > max_width {
            break;
        }

        result.push(ch);
        width += cw;
    }

    result
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
        let r = Renderer::new(80, 24, 4, false, false);
        let frame = r.render(&EditorState::new());
        assert_eq!(frame.rows.len(), 24);
    }

    #[test]
    fn test_render_empty_buffer() {
        let r = Renderer::new(80, 3, 4, false, false);
        let state = EditorState::new();
        let frame = r.render(&state);
        assert_eq!(frame.rows.len(), 3);
        assert_eq!(frame.rows[0], "");
        assert_eq!(frame.rows[1], "");
        assert!(frame.rows[2].contains("NORMAL"));
    }

    #[test]
    fn test_render_text_lines() {
        let r = Renderer::new(80, 4, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello\nworld\n!");
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "hello");
        assert_eq!(frame.rows[1], "world");
        assert_eq!(frame.rows[2], "!");
    }

    #[test]
    fn test_render_truncates_long_lines() {
        let r = Renderer::new(5, 3, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("abcdefgh");
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "abcde");
    }

    #[test]
    fn test_render_status_line_no_name() {
        let r = Renderer::new(40, 2, 4, false, false);
        let state = EditorState::new();
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("NORMAL"));
        assert!(status.contains("[No Name]"));
        assert!(status.contains("1:1"));
    }

    #[test]
    fn test_render_status_line_with_file_and_dirty() {
        let r = Renderer::new(50, 2, 4, false, false);
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
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Command);
        state.command_buffer = "write".to_string();
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], ":write");
    }

    #[test]
    fn test_cursor_position() {
        let r = Renderer::new(80, 24, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello\nworld");
        state.selection = Selection::cursor(Position::new(1, 3));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 3);
        assert_eq!(row, 1);
    }

    #[test]
    fn test_cursor_position_clamped() {
        let r = Renderer::new(5, 2, 4, false, false);
        let mut state = EditorState::new();
        state.selection = Selection::cursor(Position::new(100, 100));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 5);
        assert_eq!(row, 0);
    }

    #[test]
    fn test_render_japanese_truncated() {
        let r = Renderer::new(5, 2, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("日本語");
        let frame = r.render(&state);
        // 日本 = 4 width, 日 = 2 width, so at width 5 we can only fit 日+本 (4) but not 日+本+語 (6).
        // So it should be "日本" (width 4) since 5 >= 4 but 5 < 6.
        assert_eq!(frame.rows[0], "日本");
    }

    #[test]
    fn test_render_status_line_shows_message() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.message = Some("File saved".to_string());
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], "File saved");
    }

    #[test]
    fn test_render_search_mode() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Search);
        state.search_query = "hello".to_string();
        let frame = r.render(&state);
        assert_eq!(frame.rows[1], "/hello");
    }

    #[test]
    fn test_scroll_offset_renders_correct_lines() {
        let mut r = Renderer::new(10, 3, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("line0\nline1\nline2\nline3\nline4");
        r.scroll_offset = 2;
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "line2");
        assert_eq!(frame.rows[1], "line3");
    }

    #[test]
    fn test_ensure_cursor_visible_scrolls_down() {
        let mut r = Renderer::new(10, 3, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("line0\nline1\nline2\nline3\nline4");
        state.selection = Selection::cursor(Position::new(3, 0));
        r.ensure_cursor_visible(&state);
        assert_eq!(r.scroll_offset, 2);
    }

    #[test]
    fn test_ensure_cursor_visible_scrolls_up() {
        let mut r = Renderer::new(10, 3, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("line0\nline1\nline2\nline3\nline4");
        r.scroll_offset = 3;
        state.selection = Selection::cursor(Position::new(1, 0));
        r.ensure_cursor_visible(&state);
        assert_eq!(r.scroll_offset, 1);
    }

    #[test]
    fn test_cursor_position_with_scroll() {
        let mut r = Renderer::new(10, 5, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("line0\nline1\nline2\nline3\nline4");
        r.scroll_offset = 2;
        state.selection = Selection::cursor(Position::new(3, 2));
        let (col, row) = r.cursor_position(&state);
        assert_eq!(col, 2);
        assert_eq!(row, 1);
    }

    #[test]
    fn test_cursor_position_japanese() {
        let r = Renderer::new(80, 24, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("日本語");
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
        let r = Renderer::new(80, 24, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("a\tb\tc");
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

    #[test]
    fn test_render_empty_selection_no_highlight() {
        let r = Renderer::new(80, 2, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello world");
        state.selection = Selection::cursor(Position::new(0, 6));
        let frame = r.render(&state);
        assert_eq!(frame.rows[0], "hello world");
    }

    #[test]
    fn test_render_single_line_selection_highlighted() {
        let r = Renderer::new(80, 2, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello world");
        state.selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        let frame = r.render(&state);
        assert!(frame.rows[0].contains("\x1b[7mworld\x1b[0m"));
    }

    #[test]
    fn test_render_multi_line_selection_highlighted() {
        let r = Renderer::new(80, 3, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello\nworld\n!");
        state.selection = Selection::new(Position::new(0, 2), Position::new(1, 3));
        let frame = r.render(&state);
        assert!(frame.rows[0].contains("\x1b[7mllo\x1b[0m"));
        assert!(frame.rows[1].contains("\x1b[7mwor\x1b[0m"));
    }

    #[test]
    fn test_render_full_line_selection() {
        let r = Renderer::new(80, 3, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello\nworld");
        state.selection = Selection::new(Position::new(0, 0), Position::new(1, 5));
        let frame = r.render(&state);
        assert!(frame.rows[0].starts_with("\x1b[7mhello\x1b[0m"));
        assert!(frame.rows[1].starts_with("\x1b[7mworld\x1b[0m"));
    }

    #[test]
    fn test_render_selection_truncated_with_ansi() {
        let r = Renderer::new(8, 2, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello world");
        state.selection = Selection::new(Position::new(0, 0), Position::new(0, 11));
        let frame = r.render(&state);
        // Should truncate to 8 display columns while preserving ANSI codes
        assert!(frame.rows[0].contains("\x1b[7m"));
        // The visible portion should not exceed width
        let visible_len = frame.rows[0]
            .replace("\x1b[7m", "")
            .replace("\x1b[0m", "")
            .chars()
            .count();
        assert!(visible_len <= 8, "visible_len={}, row={}", visible_len, frame.rows[0]);
    }

    #[test]
    fn test_truncate_ansi_to_width_skips_ansi() {
        let input = "\x1b[7mhello\x1b[0m";
        let result = truncate_ansi_to_width(input, 3, 4);
        // Should include ANSI codes but only 3 visible chars
        assert!(result.contains("\x1b[7m"));
        assert!(result.contains("hel"));
        assert!(!result.contains("lo"));
    }

    #[test]
    fn test_render_backward_selection() {
        let r = Renderer::new(80, 2, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello world");
        state.selection = Selection::new(Position::new(0, 11), Position::new(0, 6));
        let frame = r.render(&state);
        assert!(frame.rows[0].contains("\x1b[7mworld\x1b[0m"));
    }

    #[test]
    fn test_selection_range_on_line_empty() {
        let sel = Selection::cursor(Position::new(0, 5));
        assert_eq!(selection_range_on_line(&sel, 0, 10), None);
    }

    #[test]
    fn test_selection_range_on_line_no_overlap() {
        let sel = Selection::new(Position::new(1, 0), Position::new(1, 5));
        assert_eq!(selection_range_on_line(&sel, 0, 10), None);
        assert_eq!(selection_range_on_line(&sel, 2, 10), None);
    }

    #[test]
    fn test_selection_range_on_line_single_line() {
        let sel = Selection::new(Position::new(0, 2), Position::new(0, 5));
        assert_eq!(selection_range_on_line(&sel, 0, 10), Some((2, 5)));
    }

    #[test]
    fn test_selection_range_on_line_spanning_start() {
        let sel = Selection::new(Position::new(0, 3), Position::new(1, 2));
        assert_eq!(selection_range_on_line(&sel, 0, 10), Some((3, 10)));
    }

    #[test]
    fn test_selection_range_on_line_spanning_end() {
        let sel = Selection::new(Position::new(0, 3), Position::new(1, 2));
        assert_eq!(selection_range_on_line(&sel, 1, 10), Some((0, 2)));
    }

    // --- Line numbers tests ---

    #[test]
    fn test_render_with_line_numbers() {
        let r = Renderer::new(20, 3, 4, true, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello\nworld");
        let frame = r.render(&state);
        assert!(frame.rows[0].starts_with("1 "));
        assert!(frame.rows[0].contains("hello"));
        assert!(frame.rows[1].starts_with("2 "));
        assert!(frame.rows[1].contains("world"));
    }

    #[test]
    fn test_render_without_line_numbers() {
        let r = Renderer::new(20, 2, 4, false, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello");
        let frame = r.render(&state);
        assert!(!frame.rows[0].starts_with("1 "));
        assert_eq!(frame.rows[0], "hello");
    }

    #[test]
    fn test_render_relative_line_numbers() {
        let r = Renderer::new(20, 4, 4, true, true);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("a\nb\nc");
        state.selection = Selection::cursor(Position::new(1, 0));
        let frame = r.render(&state);
        // Line 0 is 1 away from cursor, should show relative
        assert!(frame.rows[0].starts_with("1 "));
        // Line 1 is cursor line, should show absolute
        assert!(frame.rows[1].starts_with("2 "));
        // Line 2 is 1 away from cursor, should show relative
        assert!(frame.rows[2].starts_with("1 "));
    }

    #[test]
    fn test_cursor_position_with_line_numbers() {
        let r = Renderer::new(20, 2, 4, true, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("hello");
        state.selection = Selection::cursor(Position::new(0, 0));
        let (col, row) = r.cursor_position(&state);
        // Cursor at beginning of text, gutter is "1 " = 2 chars
        assert_eq!(col, 2);
        assert_eq!(row, 0);
    }

    #[test]
    fn test_gutter_width_single_digit() {
        let r = Renderer::new(20, 2, 4, true, false);
        let mut state = EditorState::new();
        state.buffer = LineBuffer::from("a\nb\nc");
        assert_eq!(r.compute_gutter_width(&state), 2); // "3" + space
    }

    #[test]
    fn test_gutter_width_double_digit() {
        let r = Renderer::new(20, 2, 4, true, false);
        let mut state = EditorState::new();
        let lines: Vec<String> = (0..15).map(|i| format!("line{}", i)).collect();
        state.buffer = LineBuffer::from(lines.join("\n").as_str());
        assert_eq!(r.compute_gutter_width(&state), 3); // "15" + space
    }

    #[test]
    fn test_render_status_line_skk_hiragana() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Insert);
        state.skk_enabled = true;
        state.skk_engine.state = crate::skk::SkkState::Hiragana;
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("INSERT"));
        assert!(status.contains("[あ]"));
    }

    #[test]
    fn test_render_status_line_skk_katakana() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Insert);
        state.skk_enabled = true;
        state.skk_engine.state = crate::skk::SkkState::Katakana;
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("INSERT"));
        assert!(status.contains("[ア]"));
    }

    #[test]
    fn test_render_status_line_skk_converting() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Insert);
        state.skk_enabled = true;
        state.skk_engine.state = crate::skk::SkkState::Converting;
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("INSERT"));
        assert!(status.contains("[変換]"));
    }

    #[test]
    fn test_render_status_line_skk_direct_no_label() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Insert);
        state.skk_enabled = true;
        state.skk_engine.state = crate::skk::SkkState::Direct;
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("INSERT"));
        // Should not contain any SKK labels
        assert!(!status.contains("[あ]"));
        assert!(!status.contains("[ア]"));
        assert!(!status.contains("[変換]"));
        assert!(!status.contains("[登録]"));
    }

    #[test]
    fn test_render_status_line_skk_disabled() {
        let r = Renderer::new(40, 2, 4, false, false);
        let mut state = EditorState::new();
        state.set_mode(EditorMode::Insert);
        state.skk_enabled = false;
        let frame = r.render(&state);
        let status = &frame.rows[1];
        assert!(status.contains("INSERT"));
        assert!(!status.contains("[あ]"));
        assert!(!status.contains("[ア]"));
        assert!(!status.contains("[変換]"));
        assert!(!status.contains("[登録]"));
    }

    #[test]
    fn test_render_status_line_truncated_when_too_long() {
        let r = Renderer::new(10, 2, 4, false, false);
        let mut state = EditorState::new();
        state.file_path = Some(std::path::PathBuf::from("/very/long/path/name.txt"));
        state.dirty = true;
        let frame = r.render(&state);
        let status = &frame.rows[1];
        // Should be truncated to fit within 10 columns
        assert!(status.len() <= 10 || !status.contains("name.txt"));
    }
}
