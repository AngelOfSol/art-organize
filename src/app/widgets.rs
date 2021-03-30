use std::fmt::Display;

use imgui::{im_str, ComboBox, ComboBoxPreviewMode, ImStr, Selectable, Ui};
use strum::IntoEnumIterator;

pub mod blob;
pub mod category;
pub mod confirm;
pub mod date;
pub mod gallery;
pub mod piece;
pub mod tag;

pub fn combo_box<T: IntoEnumIterator + Display + Eq>(
    ui: &Ui<'_>,
    label: &'_ ImStr,
    value: &T,
) -> Option<T> {
    let mut ret = None;
    ComboBox::new(label)
        .preview_mode(ComboBoxPreviewMode::Full)
        .preview_value(&im_str!("{}", value))
        .build(ui, || {
            for item in T::iter() {
                if Selectable::new(&im_str!("{}", item))
                    .selected(value == &item)
                    .build(ui)
                {
                    ret = Some(item);
                }
            }
        });
    ret
}
