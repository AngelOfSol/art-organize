use db::{
    commands::{AttachTag, EditPiece},
    CategoryId, Db, Piece, PieceId, TagId,
};
use imgui::{im_str, Ui};
use tag::ItemViewResponse;

use super::{
    confirm::confirm_delete_popup,
    date, enum_combo_box,
    tag::{self, InPieceViewResponse},
};

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
pub fn tooltip(piece_id: PieceId, db: &Db, ui: &Ui<'_>) {
    let piece = &db[piece_id];
    ui.text(&im_str!("Name: {}", piece.name));
    ui.text(&im_str!("Source Type: {}", piece.source_type));
    ui.text(&im_str!("Media Type: {}", piece.media_type));
    date::view("Date Added", &piece.added, ui);

    if let Some(price) = piece.base_price {
        ui.text(im_str!("Price: ${}", price));
    }
    if let Some(price) = piece.tip_price {
        ui.text(im_str!("Tipped: ${}", price));
    }
}

pub fn view_with_tags(
    piece_id: PieceId,
    db: &Db,
    ui: &Ui<'_>,
) -> Option<(TagId, ItemViewResponse)> {
    view(piece_id, db, ui);

    ui.separator();

    let mut categories = db
        .tags_for_piece(piece_id)
        .map(|tag| db.category_for_tag(tag))
        .flatten()
        .collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories.sort_by_key(|category_id| &db[category_id].name);

    for category_id in categories {
        ui.text(&im_str!("{}", db[category_id].name));
        let mut tags = db
            .tags_for_piece(piece_id)
            .filter(|tag_id| db.category_for_tag(*tag_id) == Some(category_id))
            .collect::<Vec<_>>();
        tags.sort_by_key(|tag_id| &db[tag_id].name);

        for tag_id in tags {
            match tag::item_view(ui, db, tag_id) {
                ItemViewResponse::None => {}
                response => return Some((tag_id, response)),
            }
        }
        ui.spacing();
    }

    ui.text(im_str!("tag"));

    let mut tags = db
        .tags_for_piece(piece_id)
        .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
        .collect::<Vec<_>>();
    tags.sort_by_key(|tag_id| &db[tag_id].name);

    for tag_id in tags {
        match tag::item_view(ui, db, tag_id) {
            ItemViewResponse::None => {}
            response => return Some((tag_id, response)),
        }
    }
    None
}

pub enum EditPieceResponse {
    None,
    Edit(EditPiece),
    Delete,
    AttachTag(AttachTag),
    RemoveTag(AttachTag),
    OpenTag(TagId),
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

    if let Some(source_type) = enum_combo_box(ui, im_str!("Source Type"), &piece.source_type) {
        return EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                source_type,
                ..piece.clone()
            },
        });
    }

    if let Some(media_type) = enum_combo_box(ui, im_str!("Media Type"), &piece.media_type) {
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

    ui.separator();

    let mut categories = db.categories().map(|(id, _)| id).collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories.sort_by_key(|category_id| &db[category_id].name);

    for category_id in categories {
        let _id = ui.push_id(&im_str!("{}", category_id));
        ui.text(&im_str!("{}", db[category_id].name));

        let mut tags = db
            .tags_for_piece(piece_id)
            .filter(|tag_id| db.category_for_tag(*tag_id) == Some(category_id))
            .collect::<Vec<_>>();
        tags.sort_by_key(|tag_id| &db[tag_id].name);

        for tag_id in tags {
            match tag::in_piece_view(ui, db, tag_id) {
                InPieceViewResponse::None => (),
                InPieceViewResponse::Open => return EditPieceResponse::OpenTag(tag_id),
                InPieceViewResponse::Remove => {
                    return EditPieceResponse::RemoveTag(AttachTag {
                        src: piece_id,
                        dest: tag_id,
                    })
                }
            }
        }

        if let Some(value) = add_tag_widget(db, piece_id, Some(category_id), ui) {
            return value;
        }

        ui.spacing();
    }
    ui.text(&im_str!("tag"));
    let mut tags = db
        .tags_for_piece(piece_id)
        .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
        .collect::<Vec<_>>();
    tags.sort_by_key(|tag_id| &db[tag_id].name);

    for tag_id in tags {
        match tag::in_piece_view(ui, db, tag_id) {
            InPieceViewResponse::None => (),
            InPieceViewResponse::Open => return EditPieceResponse::OpenTag(tag_id),
            InPieceViewResponse::Remove => {
                return EditPieceResponse::RemoveTag(AttachTag {
                    src: piece_id,
                    dest: tag_id,
                })
            }
        }
    }

    if let Some(value) = add_tag_widget(db, piece_id, None, ui) {
        return value;
    }

    if confirm_delete_popup(ui) {
        EditPieceResponse::Delete
    } else {
        EditPieceResponse::None
    }
}

fn add_tag_widget(
    db: &Db,
    piece_id: PieceId,
    category_id: Option<CategoryId>,
    ui: &Ui,
) -> Option<EditPieceResponse> {
    let mut unused_tags = db
        .tags()
        .filter(|(tag_id, _)| {
            !db.tags_for_piece(piece_id)
                .any(|piece_tag_id| piece_tag_id == *tag_id)
                && db.category_for_tag(*tag_id) == category_id
        })
        .collect::<Vec<_>>();
    unused_tags.sort_by_key(|(_, tag)| &tag.name);
    if !unused_tags.is_empty() {
        let (first_tag, _) = unused_tags[0];

        if let Some(tag_id) = super::combo_box(
            ui,
            &im_str!("Add"),
            unused_tags.into_iter().map(|(id, _)| id),
            &first_tag,
            |id| im_str!("{}", db[id].name),
        ) {
            return Some(EditPieceResponse::AttachTag(AttachTag {
                src: piece_id,
                dest: tag_id,
            }));
        }
    }
    None
}
