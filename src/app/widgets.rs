use std::fmt::Display;

use imgui::{im_str, ComboBox, ComboBoxPreviewMode, ImStr, ImString, Selectable, Ui};
use strum::IntoEnumIterator;

pub mod blob;
pub mod category;
pub mod confirm;
pub mod date;
pub mod gallery;
pub mod piece;
pub mod tag;

pub fn enum_combo_box<T: IntoEnumIterator + Display + Eq>(
    ui: &Ui<'_>,
    label: &'_ ImStr,
    value: &T,
) -> Option<T> {
    combo_box(ui, label, T::iter(), value, |item| item.to_string().into())
}

pub fn combo_box<I: Iterator<Item = T>, T: Eq, F: Fn(&T) -> ImString>(
    ui: &Ui<'_>,
    label: &'_ ImStr,
    items: I,
    value: &T,
    f: F,
) -> Option<T> {
    let mut ret = None;
    ComboBox::new(label)
        .preview_mode(ComboBoxPreviewMode::Full)
        .preview_value(&im_str!("{}", f(value)))
        .build(ui, || {
            for item in items {
                if Selectable::new(&im_str!("{}", f(value)))
                    .selected(value == &item)
                    .build(ui)
                {
                    ret = Some(item);
                }
            }
        });
    ret
}
