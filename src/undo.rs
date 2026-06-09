use crate::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    Insert { pos: Position, text: String },
    Delete { pos: Position, text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Transaction {
    pub ops: Vec<EditOp>,
}

impl Transaction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ops(ops: Vec<EditOp>) -> Self {
        Self { ops }
    }

    pub fn push(&mut self, op: EditOp) {
        self.ops.push(op);
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UndoManager {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
}

impl UndoManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, transaction: Transaction) {
        if !transaction.is_empty() {
            self.undo_stack.push(transaction);
            self.redo_stack.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Option<Transaction> {
        if let Some(txn) = self.undo_stack.pop() {
            self.redo_stack.push(txn.clone());
            Some(txn)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Transaction> {
        if let Some(txn) = self.redo_stack.pop() {
            self.undo_stack.push(txn.clone());
            Some(txn)
        } else {
            None
        }
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new_is_empty() {
        let txn = Transaction::new();
        assert!(txn.is_empty());
    }

    #[test]
    fn test_transaction_with_ops() {
        let txn = Transaction::with_ops(vec![
            EditOp::Insert {
                pos: Position::new(0, 0),
                text: "a".to_string(),
            },
        ]);
        assert!(!txn.is_empty());
        assert_eq!(txn.ops.len(), 1);
    }

    #[test]
    fn test_undo_manager_push_and_undo() {
        let mut um = UndoManager::new();
        assert!(!um.can_undo());

        um.push(Transaction::with_ops(vec![EditOp::Insert {
            pos: Position::new(0, 0),
            text: "hello".to_string(),
        }]));

        assert!(um.can_undo());
        assert_eq!(um.undo_len(), 1);

        let undone = um.undo();
        assert!(undone.is_some());
        assert!(!um.can_undo());
        assert!(um.can_redo());
        assert_eq!(um.redo_len(), 1);
    }

    #[test]
    fn test_undo_manager_redo() {
        let mut um = UndoManager::new();
        um.push(Transaction::with_ops(vec![EditOp::Delete {
            pos: Position::new(1, 0),
            text: "world".to_string(),
        }]));

        um.undo();
        assert!(um.can_redo());

        let redone = um.redo();
        assert!(redone.is_some());
        assert!(um.can_undo());
        assert!(!um.can_redo());
    }

    #[test]
    fn test_undo_manager_push_clears_redo() {
        let mut um = UndoManager::new();
        um.push(Transaction::with_ops(vec![EditOp::Insert {
            pos: Position::new(0, 0),
            text: "first".to_string(),
        }]));
        um.undo();
        assert!(um.can_redo());

        um.push(Transaction::with_ops(vec![EditOp::Insert {
            pos: Position::new(0, 0),
            text: "second".to_string(),
        }]));
        assert!(!um.can_redo());
    }

    #[test]
    fn test_undo_manager_empty_undo_returns_none() {
        let mut um = UndoManager::new();
        assert_eq!(um.undo(), None);
    }

    #[test]
    fn test_undo_manager_empty_redo_returns_none() {
        let mut um = UndoManager::new();
        assert_eq!(um.redo(), None);
    }

    #[test]
    fn test_undo_manager_len_tracks_correctly() {
        let mut um = UndoManager::new();
        assert_eq!(um.undo_len(), 0);
        assert_eq!(um.redo_len(), 0);

        um.push(Transaction::with_ops(vec![EditOp::Insert {
            pos: Position::new(0, 0),
            text: "a".to_string(),
        }]));
        assert_eq!(um.undo_len(), 1);
        assert_eq!(um.redo_len(), 0);

        um.push(Transaction::with_ops(vec![EditOp::Insert {
            pos: Position::new(0, 0),
            text: "b".to_string(),
        }]));
        assert_eq!(um.undo_len(), 2);
        assert_eq!(um.redo_len(), 0);

        um.undo();
        assert_eq!(um.undo_len(), 1);
        assert_eq!(um.redo_len(), 1);

        um.undo();
        assert_eq!(um.undo_len(), 0);
        assert_eq!(um.redo_len(), 2);

        um.redo();
        assert_eq!(um.undo_len(), 1);
        assert_eq!(um.redo_len(), 1);
    }
}
