use crate::{backend::DbBackend, frontend::Frontend};

pub mod edit_blob;
pub mod edit_piece;
pub mod gallery;
pub mod view_blob;
pub mod view_piece;

pub enum ViewResponse {
    Push(Box<dyn View>),
    Replace(Box<dyn View>),
    Pop,
    Unchanged,
}

impl Clone for ViewResponse {
    fn clone(&self) -> Self {
        match self {
            Self::Push(arg0) => Self::Push(arg0.boxed_clone()),
            Self::Replace(arg0) => Self::Replace(arg0.boxed_clone()),
            Self::Pop => Self::Pop,
            Self::Unchanged => Self::Unchanged,
        }
    }
}

impl ViewResponse {
    pub fn push(&mut self, view: impl View + 'static) {
        *self = Self::Push(Box::new(view));
    }

    pub fn replace(&mut self, view: impl View + 'static) {
        *self = Self::Replace(Box::new(view));
    }

    pub fn pop(&mut self) {
        *self = Self::Pop;
    }
}
pub trait View: Send + Sync {
    fn boxed_clone(&self) -> Box<dyn View>;
    fn name(&self) -> String;

    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend);
    fn side_panels(&mut self, _: &egui::CtxRef, _: &mut Frontend, _: &mut DbBackend) {}
}
