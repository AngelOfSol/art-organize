use db::{
    commands::{AttachCategory, EditTag},
    Category, Db, Tag, TagId,
};
use imgui::{im_str, Selectable, StyleColor, Ui};

use super::{combo_box, confirm::confirm_delete_popup, date};

pub fn view(tag_id: TagId, db: &Db, ui: &Ui<'_>) {
    let tag = &db[tag_id];
    ui.text_wrapped(&im_str!("Name: {}", tag.name));
    ui.text_wrapped(&im_str!("Description: {}", tag.description));
    date::view("Date Added", &tag.added, ui);

    if let Some(category_id) = db.category_for_tag(tag_id) {
        let category = &db[category_id];
        ui.text(&im_str!("Category: "));
        ui.same_line();
        ui.text_colored(category.raw_color(), im_str!("{}", category.name));
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
    Info,
    Add,
    AddNegated,
    Open,
}

pub fn item_view(ui: &Ui, db: &Db, tag_id: TagId) -> ItemViewResponse {
    let tag = &db[tag_id];

    let button_size = [ui.text_line_height_with_spacing(); 2];
    let label = im_str!("{}", tag.name);
    let _id = ui.push_id(&label);

    if Selectable::new(im_str!("?")).size(button_size).build(ui) {
        return ItemViewResponse::Info;
    }
    if Selectable::new(im_str!("+")).size(button_size).build(ui) {
        return ItemViewResponse::Add;
    }
    if Selectable::new(im_str!("!")).size(button_size).build(ui) {
        return ItemViewResponse::AddNegated;
    }

    let color = if let Some(category_id) = db.category_for_tag(tag_id) {
        db[category_id].raw_color()
    } else {
        ui.style_color(StyleColor::Text)
    };
    let _color = ui.push_style_color(StyleColor::Text, color);
    if Selectable::new(&label).build(ui) {
        ItemViewResponse::Open
    } else {
        ItemViewResponse::None
    }
}

pub enum InPieceViewResponse {
    None,
    Info,
    Open,
    Remove,
}
pub fn in_piece_view(ui: &Ui, db: &Db, tag_id: TagId) -> InPieceViewResponse {
    let tag = &db[tag_id];

    let button_size = [ui.text_line_height_with_spacing(); 2];
    let label = im_str!("{}", tag.name);
    let _id = ui.push_id(&label);

    if Selectable::new(im_str!("?")).size(button_size).build(ui) {
        return InPieceViewResponse::Info;
    }

    if Selectable::new(im_str!("-")).size(button_size).build(ui) {
        return InPieceViewResponse::Remove;
    }

    let color = if let Some(category_id) = db.category_for_tag(tag_id) {
        db[category_id].raw_color()
    } else {
        ui.style_color(StyleColor::Text)
    };
    let _color = ui.push_style_color(StyleColor::Text, color);
    if Selectable::new(&label).build(ui) {
        InPieceViewResponse::Open
    } else {
        InPieceViewResponse::None
    }
}
