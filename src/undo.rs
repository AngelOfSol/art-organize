use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

pub struct UndoStack<T> {
    history: Vec<T>,
    current: usize,
}

impl<T> Deref for UndoStack<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.history[self.current]
    }
}

impl<T> DerefMut for UndoStack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.history[self.current]
    }
}

pub struct Transaction<'a, T: Clone> {
    data: T,
    committed: bool,
    inner: &'a mut UndoStack<T>,
}

pub struct TransactionCommit {
    committed: Cell<bool>,
}

impl TransactionCommit {
    pub fn commit(&self) {
        self.committed.set(true);
    }
}

// can optimize this by making it provide a mapped type
// and only cloning the value of the mapped type
impl<'a, T: Clone> Transaction<'a, T> {
    #[must_use]
    pub fn run<F: FnOnce(&TransactionCommit, &'_ mut T)>(mut self, transaction: F) -> Self {
        let commit = TransactionCommit {
            committed: Cell::new(false),
        };
        transaction(&commit, &mut self.data);
        self.committed = commit.committed.get();

        self
    }

    pub fn finish(self) {
        if self.committed {
            self.inner.new_checkpoint(self.data);
        }
    }
}

impl<'a, T: Clone> Deref for Transaction<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: Clone> DerefMut for Transaction<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: Clone> UndoStack<T> {
    pub fn new(value: T) -> Self {
        Self {
            history: vec![value],
            current: 0,
        }
    }

    pub fn undo_checkpoint(&mut self) {
        self.new_checkpoint(self.history.last().unwrap().clone());
    }
    fn new_checkpoint(&mut self, value: T) {
        if self.current > 100 {
            self.history.remove(0);
        } else if self.current < self.history.len() - 1 {
            self.history.truncate(self.current + 1);
            self.current += 1;
        } else {
            self.current += 1;
        }
        self.history.push(value);
    }

    pub fn undo(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    pub fn can_undo(&self) -> bool {
        self.current != 0
    }

    pub fn redo(&mut self) {
        self.current = (self.current + 1).min(self.history.len() - 1);
    }

    pub fn can_redo(&self) -> bool {
        self.current != self.history.len() - 1
    }

    pub fn transaction(&mut self) -> Transaction<'_, T> {
        Transaction {
            data: self.clone(),
            committed: false,
            inner: self,
        }
    }
}

#[test]
fn test_undo_stack() {
    let mut new = UndoStack::new(0);
    new.undo_checkpoint();
    *new = 4;
    assert_eq!(*new, 4);
    assert_eq!(new.history.len(), 2);
    new.undo();
    assert_eq!(*new, 0);
    assert_eq!(new.history.len(), 2);
    new.redo();
    assert_eq!(*new, 4);
    assert_eq!(new.history.len(), 2);

    new.undo_checkpoint();
    assert_eq!(new.history.len(), 3);
    new.undo();
    new.undo();
    new.undo_checkpoint();
    assert_eq!(new.history.len(), 2);
}
