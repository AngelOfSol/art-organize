use db::{Tag, TagCategory};
use imgui::{im_str, ImStr, Selectable, StyleColor, Ui};

pub enum TagResponse {
    None,
    Info,
    Add,
    AddNegated,
    Remove,
    ReplaceSearch,
}

pub fn view(ui: &Ui, t: &Tag, tg: &TagCategory) -> TagResponse {
    let button_size = [ui.text_line_height_with_spacing(); 2];
    let label = im_str!("{}", t.name);
    let _id = ui.push_id(&label);

    if Selectable::new(im_str!("?")).size(button_size).build(ui) {
        return TagResponse::Info;
    }
    ui.same_line();

    let _color = ui.push_style_color(StyleColor::Text, tg.raw_color());
    if Selectable::new(&label).build(ui) {
        TagResponse::ReplaceSearch
    } else {
        TagResponse::None
    }
}

pub fn edit(ui: &Ui, t: &Tag, tg: &TagCategory) -> TagResponse {
    let button_size = [ui.text_line_height_with_spacing(); 2];
    let label = im_str!("{}", t.name);

    let _id = ui.push_id(&label);

    if Selectable::new(im_str!("?")).size(button_size).build(ui) {
        return TagResponse::Info;
    }
    ui.same_line();

    if Selectable::new(im_str!("-##Remove"))
        .size(button_size)
        .build(ui)
    {
        return TagResponse::Remove;
    }

    ui.same_line();

    let _color = ui.push_style_color(StyleColor::Text, tg.raw_color());
    if Selectable::new(&label).build(ui) {
        TagResponse::ReplaceSearch
    } else {
        TagResponse::None
    }
}

pub fn gallery(ui: &Ui, t: &Tag, tg: &TagCategory) -> TagResponse {
    let button_size = [ui.text_line_height_with_spacing(); 2];
    let label = im_str!("{}", t.name);

    let _id = ui.push_id(&label);

    if Selectable::new(im_str!("?")).size(button_size).build(ui) {
        return TagResponse::Info;
    }
    ui.same_line();

    if Selectable::new(im_str!("+")).size(button_size).build(ui) {
        return TagResponse::Add;
    }
    ui.same_line();
    if Selectable::new(im_str!("-##Add Negated"))
        .size(button_size)
        .build(ui)
    {
        return TagResponse::AddNegated;
    }

    ui.same_line();

    let _color = ui.push_style_color(StyleColor::Text, tg.raw_color());
    if Selectable::new(&label).build(ui) {
        TagResponse::ReplaceSearch
    } else {
        TagResponse::None
    }
}
