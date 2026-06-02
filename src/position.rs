use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.col.cmp(&other.col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(3, 5);
        assert_eq!(pos.line, 3);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_position_equality() {
        assert_eq!(Position::new(1, 2), Position::new(1, 2));
        assert_ne!(Position::new(1, 2), Position::new(2, 1));
    }

    #[test]
    fn test_position_ordering_same_line() {
        assert!(Position::new(0, 3) < Position::new(0, 5));
        assert!(Position::new(0, 5) > Position::new(0, 3));
    }

    #[test]
    fn test_position_ordering_different_line() {
        assert!(Position::new(1, 0) < Position::new(2, 0));
        assert!(Position::new(3, 100) > Position::new(2, 0));
    }

    #[test]
    fn test_position_sorting() {
        let mut positions = vec![
            Position::new(2, 5),
            Position::new(0, 0),
            Position::new(1, 10),
            Position::new(0, 5),
        ];
        positions.sort();
        assert_eq!(
            positions,
            vec![
                Position::new(0, 0),
                Position::new(0, 5),
                Position::new(1, 10),
                Position::new(2, 5),
            ]
        );
    }
}
