use db::{
    commands::{AttachCategory, EditTag},
    Db, Tag, TagId,
};
use imgui::{im_str, Selectable, StyleColor, Ui};

use super::{category, combo_box, confirm::confirm_delete_popup, date};

pub fn view(tag_id: TagId, db: &Db, ui: &Ui<'_>) {
    let tag = &db[tag_id];
    ui.text_wrapped(&im_str!("Name: {}", tag.name));
    ui.text_wrapped(&im_str!("Description: {}", tag.description));
    date::view("Date Added", &tag.added, ui);

    if let Some(category_id) = db.category_for_tag(tag_id) {
        category::link(ui, &db[category_id]);
    }
}

pub enum EditTagResponse {
    None,
    Delete,
    Edit(EditTag),
    AttachCategory(AttachCategory),
}
pub fn edit(tag_id: TagId, db: &Db, ui: &Ui<'_>) -> EditTagResponse {
    let tag = &db[tag_id];

    let mut buf = tag.name.clone().into();
    ui.input_text(im_str!("Name"), &mut buf)
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        return EditTagResponse::Edit(EditTag {
            id: tag_id,
            data: Tag {
                name: buf.to_string(),
                ..tag.clone()
            },
        });
    }

    let mut buf = tag.description.clone().into();
    ui.input_text_multiline(im_str!("Description"), &mut buf, [0.0, 100.0])
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        return EditTagResponse::Edit(EditTag {
            id: tag_id,
            data: Tag {
                description: buf.to_string(),
                ..tag.clone()
            },
        });
    }

    if let Some(added) = date::edit(im_str!("Date Added"), &tag.added, ui) {
        return EditTagResponse::Edit(EditTag {
            id: tag_id,
            data: Tag {
                added,
                ..tag.clone()
            },
        });
    }
    let x = std::iter::once(None).chain(db.categories().map(|(id, _)| Some(id)));

    if let Some(new_id) = combo_box(
        ui,
        &im_str!("Category"),
        x,
        &db.category_for_tag(tag_id),
        |id| match id {
            Some(category_id) => im_str!("{}", &db[category_id].name),
            None => Default::default(),
        },
    ) {
        return EditTagResponse::AttachCategory(AttachCategory {
            src: tag_id,
            dest: new_id,
        });
    }

    if ui.button(im_str!("Delete")) {
        ui.open_popup(im_str!("Confirm Delete"));
    }
    if confirm_delete_popup(ui) {
        EditTagResponse::Delete
    } else {
        EditTagResponse::None
    }
}

pub enum ItemViewResponse {
    None,
    Add,
    AddNegated,
    Open,
}

pub fn item_view(ui: &Ui, db: &Db, tag_id: TagId) -> ItemViewResponse {
    item_view_with_count(ui, db, tag_id, db.pieces_for_tag(tag_id).count())
}

pub fn item_view_with_count(ui: &Ui, db: &Db, tag_id: TagId, count: usize) -> ItemViewResponse {
    let tag = &db[tag_id];

    let button_size = [
        ui.calc_text_size(&im_str!("+"))[0],
        ui.text_line_height_with_spacing(),
    ];
    let label = im_str!("{}", tag.name);
    let _id = ui.push_id(&label);

    if Selectable::new(im_str!("+")).size(button_size).build(ui) {
        return ItemViewResponse::Add;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(&im_str!("Unimplemented."));
    }
    ui.same_line();
    if Selectable::new(im_str!("!")).size(button_size).build(ui) {
        return ItemViewResponse::AddNegated;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(&im_str!("Unimplemented."));
    }
    ui.same_line();
    let result = {
        let color = if let Some(category_id) = db.category_for_tag(tag_id) {
            db[category_id].raw_color()
        } else {
            ui.style_color(StyleColor::Text)
        };
        let _color = ui.push_style_color(StyleColor::Text, color);
        if Selectable::new(&label)
            .size([0.0, ui.text_line_height_with_spacing()])
            .build(ui)
        {
            ItemViewResponse::Open
        } else {
            ItemViewResponse::None
        }
    };
    if ui.is_item_hovered() && tag.description.chars().any(|c| !c.is_whitespace()) {
        ui.tooltip(|| ui.text(&im_str!("{}", tag.description)));
    }

    ui.same_line();
    ui.text_colored([0.4, 0.4, 0.4, 1.0], &im_str!("{}", count));

    result
}

pub enum InPieceViewResponse {
    None,
    Open,
    Remove,
}
pub fn in_piece_view(ui: &Ui, db: &Db, tag_id: TagId) -> InPieceViewResponse {
    let tag = &db[tag_id];

    let button_size = [ui.text_line_height_with_spacing(); 2];
    let label = im_str!("{}", tag.name);
    let _id = ui.push_id(&label);
    if Selectable::new(im_str!("-")).size(button_size).build(ui) {
        return InPieceViewResponse::Remove;
    }
    ui.same_line();

    let color = if let Some(category_id) = db.category_for_tag(tag_id) {
        db[category_id].raw_color()
    } else {
        ui.style_color(StyleColor::Text)
    };

    let result = {
        let _color = ui.push_style_color(StyleColor::Text, color);
        if Selectable::new(&label).build(ui) {
            InPieceViewResponse::Open
        } else {
            InPieceViewResponse::None
        }
    };

    if ui.is_item_hovered() && tag.description.chars().any(|c| !c.is_whitespace()) {
        ui.tooltip(|| ui.text(&im_str!("{}", tag.description)));
    }

    result
}
