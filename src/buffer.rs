use crate::position::Position;

pub struct LineBuffer {
    lines: Vec<String>,
}

impl LineBuffer {
    pub fn new() -> Self {
        todo!()
    }

    pub fn from_str(text: &str) -> Self {
        todo!()
    }

    pub fn insert(&mut self, pos: Position, text: &str) -> Position {
        todo!()
    }

    pub fn delete_range(&mut self, start: Position, end: Position) -> String {
        todo!()
    }

    pub fn line(&self, line: usize) -> Option<&str> {
        todo!()
    }

    pub fn line_count(&self) -> usize {
        todo!()
    }

    pub fn to_string(&self) -> String {
        todo!()
    }

    pub fn line_len(&self, line: usize) -> Option<usize> {
        todo!()
    }

    pub fn display_width(&self, pos: Position, tab_width: usize) -> Option<usize> {
        todo!()
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
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line(0), Some("a"));
        assert_eq!(buf.line(1), Some("c"));
        assert_eq!(buf.to_string(), "a\nc");
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
