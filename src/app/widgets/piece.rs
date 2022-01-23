use db::{
    v2::commands::{AttachTag, EditPiece},
    CategoryId, Db, Piece, PieceId, TagId,
};
use imgui::{im_str, Ui};
use tag::ItemViewResponse;

use super::{
    confirm::confirm_delete_popup,
    date, enum_combo_box,
    tag::{self, InPieceViewResponse},
};
use itertools::Itertools;

pub fn view(piece_id: PieceId, db: &Db, ui: &Ui<'_>) {
    let piece = &db[piece_id];
    ui.text_wrapped(&im_str!("Description: {}", piece.description));
    date::view("Date Added", &piece.added, ui);
}
pub fn tooltip(piece_id: PieceId, db: &Db, ui: &Ui<'_>) {
    let piece = &db[piece_id];
    date::view("Date Added", &piece.added, ui);
}
pub fn view_tags(piece_id: PieceId, db: &Db, ui: &Ui<'_>) -> Option<(TagId, ItemViewResponse)> {
    for category_id in db
        .tags_for_piece(piece_id)
        .flat_map(|tag| db.category_for_tag(tag))
        .sorted_by_key(|category_id| &db[category_id].name)
        .dedup()
    {
        ui.text(&im_str!("{}", db[category_id].name));

        for tag_id in db
            .tags_for_piece(piece_id)
            .filter(|tag_id| db.category_for_tag(*tag_id) == Some(category_id))
            .sorted_by_key(|tag_id| &db[tag_id].name)
        {
            match tag::item_view(ui, db, tag_id) {
                ItemViewResponse::None => {}
                response => return Some((tag_id, response)),
            }
        }
        ui.spacing();
    }

    ui.text(im_str!("tag"));

    for tag_id in db
        .tags_for_piece(piece_id)
        .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
        .sorted_by_key(|tag_id| &db[tag_id].name)
    {
        match tag::item_view(ui, db, tag_id) {
            ItemViewResponse::None => {}
            response => return Some((tag_id, response)),
        }
    }
    None
}

pub enum EditPieceResponse {
    Edit(EditPiece),
    Delete,
    AttachTag(AttachTag),
    RemoveTag(AttachTag),
    OpenTag(TagId),
}

#[allow(clippy::or_fun_call)]
pub fn edit_tags(piece_id: PieceId, db: &Db, ui: &Ui<'_>) -> Option<EditPieceResponse> {
    let mut ret = None;

    for category_id in db
        .categories()
        .map(|(id, _)| id)
        .sorted_by_key(|category_id| &db[category_id].name)
        .dedup()
    {
        let _id = ui.push_id(&im_str!("{}", category_id));
        ui.text(&im_str!("{}", db[category_id].name));

        for tag_id in db
            .tags_for_piece(piece_id)
            .filter(|tag_id| db.category_for_tag(*tag_id) == Some(category_id))
            .sorted_by_key(|tag_id| &db[tag_id].name)
        {
            match tag::in_piece_view(ui, db, tag_id) {
                InPieceViewResponse::None => (),
                InPieceViewResponse::Open => {
                    ret.get_or_insert(EditPieceResponse::OpenTag(tag_id));
                }
                InPieceViewResponse::Remove => {
                    ret.get_or_insert(EditPieceResponse::RemoveTag(AttachTag {
                        src: piece_id,
                        dest: tag_id,
                    }));
                }
            }
        }

        ret = ret.or(add_tag_widget(db, piece_id, Some(category_id), ui));

        ui.spacing();
    }
    ui.text(&im_str!("tag"));

    for tag_id in db
        .tags_for_piece(piece_id)
        .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
        .sorted_by_key(|tag_id| &db[tag_id].name)
    {
        match tag::in_piece_view(ui, db, tag_id) {
            InPieceViewResponse::None => (),
            InPieceViewResponse::Open => {
                ret.get_or_insert(EditPieceResponse::OpenTag(tag_id));
            }
            InPieceViewResponse::Remove => {
                ret.get_or_insert(EditPieceResponse::RemoveTag(AttachTag {
                    src: piece_id,
                    dest: tag_id,
                }));
            }
        }
    }

    ret = ret.or(add_tag_widget(db, piece_id, None, ui));

    ret
}

pub fn edit(piece_id: PieceId, db: &Db, ui: &Ui<'_>) -> Option<EditPieceResponse> {
    let mut ret = None;

    let piece = &db[piece_id];

    let mut buf = piece.description.clone().into();
    ui.input_text(im_str!("Description"), &mut buf)
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        ret.get_or_insert(EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                description: buf.to_string(),
                ..piece.clone()
            },
        }));
    }

    if let Some(date) = date::edit(im_str!("Date Added"), &piece.added, ui) {
        ret.get_or_insert(EditPieceResponse::Edit(EditPiece {
            id: piece_id,
            data: Piece {
                added: date,
                ..piece.clone()
            },
        }));
    }

    if ui.button(im_str!("Delete")) {
        ui.open_popup(im_str!("Confirm Delete"));
    }

    if confirm_delete_popup(ui) {
        ret.get_or_insert(EditPieceResponse::Delete);
    }

    ret
}

fn add_tag_widget(
    db: &Db,
    piece_id: PieceId,
    category_id: Option<CategoryId>,
    ui: &Ui,
) -> Option<EditPieceResponse> {
    let unused_tags = db
        .tags()
        .filter(|(tag_id, _)| {
            !db.tags_for_piece(piece_id)
                .any(|piece_tag_id| piece_tag_id == *tag_id)
                && db.category_for_tag(*tag_id) == category_id
        })
        .sorted_by_key(|(_, tag)| &tag.name)
        .collect_vec();
    if !unused_tags.is_empty() {
        let (first_tag, _) = unused_tags[0];

        if let Some(tag_id) = super::combo_box(
            ui,
            im_str!("Add"),
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
