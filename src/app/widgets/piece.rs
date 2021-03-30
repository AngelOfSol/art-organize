use crate::app::{tag, tag_category};
use chrono::Local;
use db::{commands::EditPiece, Db, Piece, PieceId, Tag, TagCategory};
use imgui::{im_str, ComboBox, ComboBoxPreviewMode, ImStr, PopupModal, Selectable, Ui};
use std::fmt::Display;
use strum::IntoEnumIterator;

use super::{combo_box, date};

pub fn view(piece_id: PieceId, db: &Db, ui: &Ui<'_>) {
    let piece = &db[piece_id];
    ui.text_wrapped(&im_str!("Name: {}", piece.name));
    ui.text_wrapped(&im_str!("Source Type: {}", piece.source_type));
    ui.text_wrapped(&im_str!("Media Type: {}", piece.media_type));
    date::view("Date Added", &piece.added, ui);

    if let Some(price) = piece.base_price {
        ui.text(im_str!("Price: ${}", price));
    }
    if let Some(price) = piece.tip_price {
        ui.text(im_str!("Tipped: ${}", price));
    }
}

pub fn view_with_tags(piece_id: PieceId, db: &Db, ui: &Ui<'_>) {
    view(piece_id, db, ui);

    ui.separator();

    for i in 0..10u32 {
        let tg = TagCategory {
            name: format!("category_{}", i),
            color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
            added: Local::today().naive_local(),
            ..TagCategory::default()
        };
        let raw_color = [
            tg.color[0] as f32 / 255.0,
            tg.color[1] as f32 / 255.0,
            tg.color[2] as f32 / 255.0,
            tg.color[3] as f32 / 255.0,
        ];

        tag_category::view(ui, &im_str!("{}", tg.name), raw_color);
        ui.indent();
        for j in 0..2 {
            let t = Tag {
                name: format!("tag_{}", j),
                description: format!("My test description {}", j),
                added: Local::today().naive_local(),
                links: Vec::new(),
            };
            tag::view(ui, &t, &tg);
        }
        ui.unindent();
    }
}

pub enum EditPieceResponse {
    None,
    Edit(EditPiece),
    Delete,
}

pub fn edit(piece_id: PieceId, db: &Db, ui: &Ui<'_>) -> EditPieceResponse {
    let piece = &db[piece_id];

    let mut buf = piece.name.clone().into();
    ui.input_text(im_str!("Name"), &mut buf)
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        return EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                name: buf.to_string(),
                ..piece.clone()
            },
        });
    }

    if let Some(source_type) = combo_box(ui, im_str!("Source Type"), &piece.source_type) {
        return EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                source_type,
                ..piece.clone()
            },
        });
    }

    if let Some(media_type) = combo_box(ui, im_str!("Media Type"), &piece.media_type) {
        return EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                media_type,
                ..piece.clone()
            },
        });
    }

    if let Some(date) = date::edit(im_str!("Date Added"), &piece.added, ui) {
        return EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                added: date,
                ..piece.clone()
            },
        });
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Format: Month/Day/Year\nHit enter to submit.");
    }

    let mut buf = piece
        .base_price
        .map(|price| price.to_string())
        .unwrap_or_else(String::new)
        .into();

    ui.input_text(im_str!("Price"), &mut buf)
        .chars_decimal(true)
        .resize_buffer(true)
        .build();
    if ui.is_item_deactivated_after_edit() {
        if buf.is_empty() {
            return EditPieceResponse::Edit(EditPiece {
                id: piece_id,
                data: Piece {
                    base_price: None,
                    ..piece.clone()
                },
            });
        } else if let Ok(base_price) = buf.to_string().parse() {
            return EditPieceResponse::Edit(EditPiece {
                id: piece_id,
                data: Piece {
                    base_price: Some(base_price),
                    ..piece.clone()
                },
            });
        }
    }

    let mut buf = piece
        .tip_price
        .map(|price| price.to_string())
        .unwrap_or_else(String::new)
        .into();
    ui.input_text(im_str!("Tip"), &mut buf)
        .chars_decimal(true)
        .resize_buffer(true)
        .build();
    if ui.is_item_deactivated_after_edit() {
        if buf.is_empty() {
            return EditPieceResponse::Edit(EditPiece {
                id: piece_id,
                data: Piece {
                    tip_price: None,
                    ..piece.clone()
                },
            });
        } else if let Ok(tip_price) = buf.to_string().parse() {
            return EditPieceResponse::Edit(EditPiece {
                id: piece_id,
                data: Piece {
                    tip_price: Some(tip_price),
                    ..piece.clone()
                },
            });
        }
    }

    if ui.button(im_str!("Delete")) {
        ui.open_popup(im_str!("Confirm Delete"));
    }

    // TODO fix this up to the actual tags

    ui.separator();

    for i in 0..10u32 {
        let tg = TagCategory {
            name: format!("category_{}", i),
            color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
            added: Local::today().naive_local(),
            ..TagCategory::default()
        };

        for j in 0..2 {
            let t = Tag {
                name: format!("tag_{}", j),
                description: format!("My test description {}", j),
                added: Local::today().naive_local(),
                links: Vec::new(),
            };
            tag::edit(ui, &t, &tg);
        }
    }

    ui.input_text(im_str!("##Add Tag Piece Edit"), &mut Default::default())
        .hint(im_str!("Add Tag"))
        .resize_buffer(true)
        .build();

    let mut response = EditPieceResponse::None;
    PopupModal::new(im_str!("Confirm Delete"))
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .always_auto_resize(true)
        .build(ui, || {
            ui.text(im_str!("Are you sure you want to delete this?"));

            if ui.button(im_str!("Yes, delete.")) {
                response = EditPieceResponse::Delete;
                ui.close_current_popup();
            }
            ui.same_line();

            if ui.button(im_str!("Cancel")) {
                ui.close_current_popup();
            }
        });
    response
}
