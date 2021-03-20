use std::fmt::Display;

use db::{Db, PieceId, Tag, TagCategory};
use imgui::{im_str, ComboBox, ComboBoxPreviewMode, ImStr, Selectable, Ui};
use strum::IntoEnumIterator;

use crate::undo::UndoStack;

pub enum Response {}

pub fn view(piece_id: PieceId, db: &UndoStack<Db>, ui: &Ui<'_>) {}

pub fn edit(piece_id: PieceId, db: &mut UndoStack<Db>, ui: &Ui<'_>) {
    db.transaction()
        .run(|commit, db| {
            let piece = if let Some(value) = db.pieces.get_mut(piece_id) {
                value
            } else {
                return;
            };

            let mut buf = piece.name.clone().into();
            if ui
                .input_text(im_str!("Name"), &mut buf)
                .resize_buffer(true)
                .build()
            {
                piece.name = buf.to_string();
            }
            if ui.is_item_deactivated_after_edit() {
                commit.commit();
            }

            if combo_box(ui, im_str!("Source Type"), &mut piece.source_type) {
                commit.commit();
            }
            if combo_box(ui, im_str!("Media Type"), &mut piece.media_type) {
                commit.commit();
            };

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
                commit.commit();
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
                commit.commit();
            }

            // TODO fix this up to the actual tags

            ui.separator();

            for i in 0..10u32 {
                let tg = TagCategory {
                    name: format!("category_{}", i),
                    color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
                    added: chrono::Local::now(),
                };
                let raw_color = [
                    tg.color[0] as f32 / 255.0,
                    tg.color[1] as f32 / 255.0,
                    tg.color[2] as f32 / 255.0,
                    tg.color[3] as f32 / 255.0,
                ];

                for j in 0..2 {
                    let t = Tag {
                        name: format!("tag_{}", j),
                        description: format!("My test description {}", j),
                        added: chrono::Local::now(),
                        links: Vec::new(),
                    };
                    let label = im_str!("{}:{}", tg.name, t.name);
                    crate::app::tag::tag_old(
                        ui,
                        &label,
                        raw_color,
                        crate::app::tag::ExtraType::Edit,
                    );
                }
            }

            ui.input_text(im_str!("##Add Tag Piece Edit"), &mut Default::default())
                .hint(im_str!("Add Tag"))
                .resize_buffer(true)
                .build();
        })
        .finish();
}

fn combo_box<T: IntoEnumIterator + Display + Eq>(
    ui: &Ui<'_>,
    label: &'_ ImStr,
    value: &mut T,
) -> bool {
    let mut changed = false;
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
                    changed = true;
                }
            }
        });

    changed
}
