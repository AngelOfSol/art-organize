use db::{commands::EditCategory, Category, CategoryId, Db};
use imgui::{
    im_str, ColorEdit, ColorEditDisplayMode, ColorEditInputMode, ColorPicker, ImStr, Selectable, Ui,
};

use super::{confirm::confirm_delete_popup, date};

pub fn item_view(ui: &Ui, label: &ImStr, raw_color: [f32; 4]) -> bool {
    let ret = Selectable::new(im_str!("?"))
        .size([ui.text_line_height_with_spacing(); 2])
        .build(ui);
    ui.same_line();
    ui.text_colored(raw_color, label);
    ret
}

pub fn view(category_id: CategoryId, db: &Db, ui: &Ui<'_>) {
    let tag = &db[category_id];
    ui.text_wrapped(&im_str!("Name: {}", tag.name));
    ui.text_wrapped(&im_str!("Description: {}", tag.description));
    // TODO colors
    date::view("Date Added", &tag.added, ui);
}

pub enum EditCategoryResponse {
    None,
    Delete,
    Edit(EditCategory),
}
pub fn edit(category_id: CategoryId, db: &Db, ui: &Ui<'_>) -> EditCategoryResponse {
    let category = &db[category_id];

    let mut buf = category.name.clone().into();
    ui.input_text(im_str!("Name"), &mut buf)
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        return EditCategoryResponse::Edit(EditCategory {
            id: category_id,
            data: Category {
                name: buf.to_string(),
                ..category.clone()
            },
        });
    }

    let mut buf = category.description.clone().into();
    ui.input_text_multiline(im_str!("Description"), &mut buf, [0.0, 100.0])
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        return EditCategoryResponse::Edit(EditCategory {
            id: category_id,
            data: Category {
                description: buf.to_string(),
                ..category.clone()
            },
        });
    }

    if let Some(added) = date::edit(im_str!("Date Added"), &category.added, ui) {
        return EditCategoryResponse::Edit(EditCategory {
            id: category_id,
            data: Category {
                added,
                ..category.clone()
            },
        });
    }

    let mut color = category.raw_color();

    let result = if ColorEdit::new(im_str!("Color"), &mut color)
        .alpha(false)
        .input_mode(ColorEditInputMode::Rgb)
        .display_mode(ColorEditDisplayMode::Rgb)
        .format(imgui::ColorFormat::U8)
        .build(ui)
    {
        let mut new_color = [0; 4];
        for (f_value, i_value) in color.iter().zip(new_color.iter_mut()) {
            *i_value = (f_value * 255.0) as u8;
        }

        EditCategoryResponse::Edit(EditCategory {
            id: category_id,
            data: Category {
                color: new_color,
                ..category.clone()
            },
        })
    } else {
        EditCategoryResponse::None
    };

    if ui.button(im_str!("Delete")) {
        ui.open_popup(im_str!("Confirm Delete"));
    }
    if confirm_delete_popup(ui) {
        EditCategoryResponse::Delete
    } else {
        result
    }
}
