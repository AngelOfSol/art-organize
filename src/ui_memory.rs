use std::{any::Any, str::FromStr};

use chrono::NaiveDate;
use egui::{Id, Response, TextEdit, Ui, Widget, WidgetText};

pub struct TextItemEdit<'a, T> {
    data: &'a mut T,
    id: Id,
    hint_text: WidgetText,
}

impl<'a, T: 'static + Clone + TextEditable> TextItemEdit<'a, T> {
    pub fn new(id: Id, data: &'a mut T) -> Self {
        Self {
            data,
            id,
            hint_text: Default::default(),
        }
    }

    pub fn hint_text(mut self, hint_text: impl Into<WidgetText>) -> Self {
        self.hint_text = hint_text.into();
        self
    }
}

impl<'a, T: 'static + Clone + TextEditable> Widget for TextItemEdit<'a, T> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut memory = ui
            .memory()
            .data
            .get_temp(self.id)
            .unwrap_or_else(|| self.data.to_text());

        let widget = TextEdit::singleline(&mut memory).hint_text(self.hint_text);

        let response = ui.add(widget);

        if response.changed() && let Some(new_value) = T::from_text(&memory) {
            *self.data = new_value;
        }

        ui.memory().data.insert_temp(self.id, memory);

        response
    }
}

impl TextEditable for chrono::NaiveDate {
    fn to_text(&self) -> String {
        self.to_string()
    }

    fn from_text(text: &str) -> Option<Self> {
        NaiveDate::from_str(text).ok()
    }
}

impl TextEditable for String {
    fn to_text(&self) -> String {
        self.clone()
    }

    fn from_text(text: &str) -> Option<Self> {
        Some(text.to_string())
    }
}

impl TextEditable for i64 {
    fn to_text(&self) -> String {
        self.to_string()
    }

    fn from_text(text: &str) -> Option<Self> {
        text.parse().ok()
    }
}

impl<T: TextEditable> TextEditable for Option<T> {
    fn to_text(&self) -> String {
        self.as_ref()
            .map(|inner| inner.to_text())
            .unwrap_or_default()
    }

    fn from_text(text: &str) -> Option<Self> {
        if text.trim() == "" {
            Some(None)
        } else {
            Some(Some(T::from_text(text.trim())?))
        }
    }
}

pub trait TextEditable: Sized {
    fn to_text(&self) -> String;
    fn from_text(text: &str) -> Option<Self>;
}

pub trait MemoryExt {
    fn text_editable<T: TextEditable + Clone + 'static>(
        &mut self,
        id: Id,
        data: &mut T,
    ) -> Response;
    fn with_memory<
        T: 'static + Any + Clone + Send + Sync,
        R,
        F: FnOnce(&mut Ui, &mut T) -> R,
        I: FnOnce() -> T,
    >(
        &mut self,
        id: egui::Id,
        init: I,
        f: F,
    ) -> R;
}

impl MemoryExt for Ui {
    fn with_memory<
        T: 'static + Any + Clone + Send + Sync,
        R,
        F: FnOnce(&mut Ui, &mut T) -> R,
        I: FnOnce() -> T,
    >(
        &mut self,
        id: egui::Id,
        init: I,
        f: F,
    ) -> R {
        let mut value = self
            .memory()
            .data
            .get_temp_mut_or_insert_with(id, init)
            .clone();

        let result = f(self, &mut value);

        self.memory().data.insert_temp(id, value);

        result
    }

    fn text_editable<T: TextEditable + Clone + 'static>(
        &mut self,
        id: Id,
        data: &mut T,
    ) -> Response {
        self.add(TextItemEdit::new(id, data))
    }
}
