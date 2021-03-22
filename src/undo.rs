use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
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
