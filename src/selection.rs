use crate::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Position,
    pub head: Position,
}

impl Selection {
    pub fn new(anchor: Position, head: Position) -> Self {
        Self { anchor, head }
    }

    pub fn cursor(pos: Position) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Returns (start, end) where start <= end
    pub fn sorted(&self) -> (Position, Position) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    pub fn collapse(&self) -> Self {
        Self::cursor(self.head)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_cursor_is_empty() {
        let sel = Selection::cursor(Position::new(2, 5));
        assert!(sel.is_empty());
        assert_eq!(sel.anchor, Position::new(2, 5));
        assert_eq!(sel.head, Position::new(2, 5));
    }

    #[test]
    fn test_selection_non_empty() {
        let sel = Selection::new(Position::new(0, 0), Position::new(0, 5));
        assert!(!sel.is_empty());
    }

    #[test]
    fn test_selection_sorted_forward() {
        let sel = Selection::new(Position::new(1, 2), Position::new(3, 4));
        let (start, end) = sel.sorted();
        assert_eq!(start, Position::new(1, 2));
        assert_eq!(end, Position::new(3, 4));
    }

    #[test]
    fn test_selection_sorted_backward() {
        let sel = Selection::new(Position::new(3, 4), Position::new(1, 2));
        let (start, end) = sel.sorted();
        assert_eq!(start, Position::new(1, 2));
        assert_eq!(end, Position::new(3, 4));
    }

    #[test]
    fn test_selection_collapse() {
        let sel = Selection::new(Position::new(0, 0), Position::new(2, 5));
        let collapsed = sel.collapse();
        assert!(collapsed.is_empty());
        assert_eq!(collapsed.head, Position::new(2, 5));
    }
}
