use std::fmt::Display;

use db::{Db, PieceId};
use imgui::{im_str, ComboBox, ComboBoxPreviewMode, ImStr, Selectable, Ui};
use strum::IntoEnumIterator;

use super::actor::TabResult;

pub struct PieceEditor {
    pub id: PieceId,
}

impl PieceEditor {
    pub fn update(&mut self) {}
    pub fn label<'a>(&self, db: &'a Db) -> Option<&'a str> {
        db.pieces.get(self.id).map(|piece| piece.name.as_str())
    }
    pub fn render(&mut self, db: &mut Db, ui: &Ui<'_>) -> TabResult {
        let piece = if let Some(value) = db.pieces.get_mut(self.id) {
            value
        } else {
            return TabResult::Kill;
        };

        let mut buf = piece.name.clone().into();
        if ui
            .input_text(im_str!("Name"), &mut buf)
            .resize_buffer(true)
            .build()
        {
            // TODO add undo saving here
            piece.name = buf.to_string();
        }

        combo_box(ui, im_str!("Source Type"), &mut piece.source_type);
        combo_box(ui, im_str!("Media Type"), &mut piece.media_type);

        ui.text(im_str!(
            "Date Added: {}",
            piece.added.format("%-m/%-d/%-Y %-H:%-M %P")
        ));

        let mut buf = piece
            .base_price
            .map(|price| price.to_string())
            .unwrap_or_else(String::new)
            .into();
        if ui
            .input_text(im_str!("Price"), &mut buf)
            .chars_decimal(true)
            .resize_buffer(true)
            .build()
        {
            piece.base_price = buf.to_string().parse().ok();
        }

        let mut buf = piece
            .tip_price
            .map(|price| price.to_string())
            .unwrap_or_else(String::new)
            .into();
        if ui
            .input_text(im_str!("Tip"), &mut buf)
            .chars_decimal(true)
            .resize_buffer(true)
            .build()
        {
            if buf.is_empty() {
                piece.tip_price = None;
            } else if let Ok(value) = buf.to_string().parse() {
                piece.tip_price = Some(value);
            }
        }

        TabResult::Selected
    }
}

fn combo_box<T: IntoEnumIterator + Display + Eq>(ui: &Ui<'_>, label: &'_ ImStr, value: &mut T) {
    ComboBox::new(label)
        .preview_mode(ComboBoxPreviewMode::Full)
        .preview_value(&im_str!("{}", value))
        .build(ui, || {
            for item in T::iter() {
                if Selectable::new(&im_str!("{}", item))
                    .selected(value == &item)
                    .build(ui)
                {
                    *value = item;
                }
            }
        });
}
