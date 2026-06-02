use crate::position::Position;

pub struct LineBuffer {
    lines: Vec<String>,
}

impl LineBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn from_str(text: &str) -> Self {
        let normalized = text.replace("\r\n", "\n");
        let lines: Vec<String> = normalized.split('\n').map(String::from).collect();
        Self { lines }
    }

    pub fn insert(&mut self, pos: Position, text: &str) -> Position {
        let normalized = text.replace("\r\n", "\n");
        let inserted_lines: Vec<&str> = normalized.split('\n').collect();

        if pos.line >= self.lines.len() {
            return pos;
        }

        let current_line = &self.lines[pos.line];
        let byte_idx = char_idx_to_byte_idx(current_line, pos.col);

        if inserted_lines.len() == 1 {
            self.lines[pos.line].insert_str(byte_idx, inserted_lines[0]);
            Position::new(pos.line, pos.col + inserted_lines[0].chars().count())
        } else {
            let after = self.lines[pos.line].split_off(byte_idx);
            self.lines[pos.line].push_str(inserted_lines[0]);

            let last_inserted = inserted_lines[inserted_lines.len() - 1];
            let last_line = last_inserted.to_string() + &after;

            let mut new_lines: Vec<String> = inserted_lines[1..inserted_lines.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect();
            new_lines.push(last_line);

            self.lines
                .splice((pos.line + 1)..(pos.line + 1), new_lines);

            let last_line_idx = pos.line + inserted_lines.len() - 1;
            let last_col = last_inserted.chars().count();
            Position::new(last_line_idx, last_col)
        }
    }

    pub fn delete_range(&mut self, start: Position, end: Position) -> String {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        if start.line == end.line {
            let line = &mut self.lines[start.line];
            let start_byte = char_idx_to_byte_idx(line, start.col);
            let end_byte = char_idx_to_byte_idx(line, end.col);
            let deleted = line[start_byte..end_byte].to_string();
            line.replace_range(start_byte..end_byte, "");
            deleted
        } else {
            let start_line = &self.lines[start.line];
            let start_byte = char_idx_to_byte_idx(start_line, start.col);
            let start_deleted = &start_line[start_byte..];

            let end_line = &self.lines[end.line];
            let end_byte = char_idx_to_byte_idx(end_line, end.col);
            let end_deleted = &end_line[..end_byte];

            let mut deleted = String::new();
            deleted.push_str(start_deleted);
            for i in (start.line + 1)..end.line {
                deleted.push('\n');
                deleted.push_str(&self.lines[i]);
            }
            deleted.push('\n');
            deleted.push_str(end_deleted);

            let before = self.lines[start.line][..start_byte].to_string();
            let after = self.lines[end.line][end_byte..].to_string();
            self.lines[start.line] = before + &after;
            self.lines.drain((start.line + 1)..=end.line);

            deleted
        }
    }

    pub fn line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line_len(&self, line: usize) -> Option<usize> {
        self.lines.get(line).map(|s| s.chars().count())
    }

    pub fn display_width(&self, pos: Position, tab_width: usize) -> Option<usize> {
        let line = self.lines.get(pos.line)?;
        if pos.col > line.chars().count() {
            return None;
        }
        let mut width = 0;
        for (i, ch) in line.chars().enumerate() {
            if i >= pos.col {
                break;
            }
            width += char_display_width(ch, tab_width, width);
        }
        Some(width)
    }
}

fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn char_display_width(ch: char, tab_width: usize, current_width: usize) -> usize {
    if ch == '\t' {
        tab_width - (current_width % tab_width)
    } else {
        unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let buf = LineBuffer::new();
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line(0), Some(""));
        assert_eq!(buf.to_string(), "");
    }

    #[test]
    fn test_from_str_empty() {
        let buf = LineBuffer::from_str("");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line(0), Some(""));
        assert_eq!(buf.to_string(), "");
    }

    #[test]
    fn test_from_str_single_line() {
        let buf = LineBuffer::from_str("hello");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line(0), Some("hello"));
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn test_from_str_multiple_lines_lf() {
        let buf = LineBuffer::from_str("hello\nworld");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("hello"));
        assert_eq!(buf.line(1), Some("world"));
        assert_eq!(buf.to_string(), "hello\nworld");
    }

    #[test]
    fn test_from_str_crlf_normalized() {
        let buf = LineBuffer::from_str("hello\r\nworld");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("hello"));
        assert_eq!(buf.line(1), Some("world"));
        assert_eq!(buf.to_string(), "hello\nworld");
    }

    #[test]
    fn test_from_str_trailing_newline() {
        let buf = LineBuffer::from_str("a\n");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("a"));
        assert_eq!(buf.line(1), Some(""));
        assert_eq!(buf.to_string(), "a\n");
    }

    #[test]
    fn test_line_out_of_range() {
        let buf = LineBuffer::from_str("a");
        assert_eq!(buf.line(1), None);
    }

    #[test]
    fn test_line_len() {
        let buf = LineBuffer::from_str("hello");
        assert_eq!(buf.line_len(0), Some(5));
    }

    #[test]
    fn test_line_len_japanese() {
        let buf = LineBuffer::from_str("日本語");
        assert_eq!(buf.line_len(0), Some(3));
    }

    #[test]
    fn test_line_len_out_of_range() {
        let buf = LineBuffer::from_str("a");
        assert_eq!(buf.line_len(1), None);
    }

    #[test]
    fn test_insert_at_start_empty() {
        let mut buf = LineBuffer::new();
        let end = buf.insert(Position::new(0, 0), "abc");
        assert_eq!(end, Position::new(0, 3));
        assert_eq!(buf.line(0), Some("abc"));
        assert_eq!(buf.to_string(), "abc");
    }

    #[test]
    fn test_insert_at_end() {
        let mut buf = LineBuffer::from_str("hello");
        let end = buf.insert(Position::new(0, 5), " world");
        assert_eq!(end, Position::new(0, 11));
        assert_eq!(buf.to_string(), "hello world");
    }

    #[test]
    fn test_insert_mid_line() {
        let mut buf = LineBuffer::from_str("abcd");
        let end = buf.insert(Position::new(0, 2), "XY");
        assert_eq!(end, Position::new(0, 4));
        assert_eq!(buf.to_string(), "abXYcd");
    }

    #[test]
    fn test_insert_newline_splits_line() {
        let mut buf = LineBuffer::from_str("ab");
        let end = buf.insert(Position::new(0, 1), "\n");
        assert_eq!(end, Position::new(1, 0));
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("a"));
        assert_eq!(buf.line(1), Some("b"));
    }

    #[test]
    fn test_insert_multiline_text() {
        let mut buf = LineBuffer::from_str("start");
        let end = buf.insert(Position::new(0, 2), "x\ny");
        assert_eq!(end, Position::new(1, 1));
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("stx"));
        assert_eq!(buf.line(1), Some("yart"));
    }

    #[test]
    fn test_insert_crlf_normalized() {
        let mut buf = LineBuffer::from_str("ab");
        let end = buf.insert(Position::new(0, 1), "\r\n");
        assert_eq!(end, Position::new(1, 0));
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("a"));
        assert_eq!(buf.line(1), Some("b"));
    }

    #[test]
    fn test_insert_japanese() {
        let mut buf = LineBuffer::new();
        let end = buf.insert(Position::new(0, 0), "日本語");
        assert_eq!(end, Position::new(0, 3));
        assert_eq!(buf.line_len(0), Some(3));
        assert_eq!(buf.to_string(), "日本語");
    }

    #[test]
    fn test_delete_range_single_line() {
        let mut buf = LineBuffer::from_str("hello world");
        let deleted = buf.delete_range(Position::new(0, 5), Position::new(0, 11));
        assert_eq!(deleted, " world");
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn test_delete_range_mid_line() {
        let mut buf = LineBuffer::from_str("abcdef");
        let deleted = buf.delete_range(Position::new(0, 2), Position::new(0, 4));
        assert_eq!(deleted, "cd");
        assert_eq!(buf.to_string(), "abef");
    }

    #[test]
    fn test_delete_range_multiline() {
        let mut buf = LineBuffer::from_str("a\nb\nc");
        let deleted = buf.delete_range(Position::new(0, 1), Position::new(2, 0));
        assert_eq!(deleted, "\nb\n");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line(0), Some("ac"));
        assert_eq!(buf.to_string(), "ac");
    }

    #[test]
    fn test_delete_range_across_lines() {
        let mut buf = LineBuffer::from_str("hello\nworld");
        let deleted = buf.delete_range(Position::new(0, 3), Position::new(1, 2));
        assert_eq!(deleted, "lo\nwo");
        assert_eq!(buf.to_string(), "helrld");
    }

    #[test]
    fn test_delete_range_empty() {
        let mut buf = LineBuffer::from_str("abc");
        let deleted = buf.delete_range(Position::new(0, 1), Position::new(0, 1));
        assert_eq!(deleted, "");
        assert_eq!(buf.to_string(), "abc");
    }

    #[test]
    fn test_display_width_ascii() {
        let buf = LineBuffer::from_str("abc");
        assert_eq!(buf.display_width(Position::new(0, 0), 4), Some(0));
        assert_eq!(buf.display_width(Position::new(0, 2), 4), Some(2));
    }

    #[test]
    fn test_display_width_tab() {
        let buf = LineBuffer::from_str("\ta");
        assert_eq!(buf.display_width(Position::new(0, 0), 4), Some(0));
        assert_eq!(buf.display_width(Position::new(0, 1), 4), Some(4));
    }

    #[test]
    fn test_display_width_japanese() {
        let buf = LineBuffer::from_str("日本語");
        assert_eq!(buf.display_width(Position::new(0, 0), 4), Some(0));
        assert_eq!(buf.display_width(Position::new(0, 1), 4), Some(2));
        assert_eq!(buf.display_width(Position::new(0, 2), 4), Some(4));
        assert_eq!(buf.display_width(Position::new(0, 3), 4), Some(6));
    }

    #[test]
    fn test_display_width_out_of_range() {
        let buf = LineBuffer::from_str("a");
        assert_eq!(buf.display_width(Position::new(0, 5), 4), None);
        assert_eq!(buf.display_width(Position::new(1, 0), 4), None);
    }
}
