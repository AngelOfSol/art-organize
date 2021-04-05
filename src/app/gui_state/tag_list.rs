use imgui::{im_str, Selectable, StyleColor};

use super::{category::CategoryView, tag::TagView, GuiHandle, GuiView};

#[derive(Debug)]
pub struct TagList;

impl GuiView for TagList {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(&mut self, gui_handle: &GuiHandle, _: &super::InnerGuiState, ui: &imgui::Ui<'_>) {
        let db = gui_handle.db.read().unwrap();

        ui.columns(2, im_str!("header"), true);

        ui.text(im_str!("Name"));
        ui.next_column();
        ui.text(im_str!("Category"));

        ui.columns(1, im_str!("unheader"), false);
        ui.separator();
        ui.columns(2, im_str!("tag list"), true);
        for (tag_id, tag) in db.tags() {
            let _id = ui.push_id(&im_str!("{}", tag_id));
            if Selectable::new(&im_str!("{}", tag.name))
                .span_all_columns(false)
                .build(ui)
            {
                gui_handle.goto(TagView {
                    id: tag_id,
                    edit: false,
                })
            }
            ui.same_line();
            ui.text_colored(
                [0.4, 0.4, 0.4, 1.0],
                im_str!("{}", db.pieces_for_tag(tag_id).count()),
            );
            ui.next_column();

            if let Some(category_id) = db.category_for_tag(tag_id) {
                {
                    let _color = ui.push_style_color(StyleColor::Text, db[category_id].raw_color());
                    if Selectable::new(&im_str!("{}", db[category_id].name))
                        .span_all_columns(false)
                        .build(ui)
                    {
                        gui_handle.goto(CategoryView {
                            id: category_id,
                            edit: false,
                        });
                    }
                }
                ui.same_line();
                ui.text_colored(
                    [0.4, 0.4, 0.4, 1.0],
                    im_str!("{}", db.tags_for_category(category_id).count()),
                );
            }
            ui.next_column();
        }

        ui.columns(1, im_str!("untag list"), false);
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        if ui.button(im_str!("New Tag")) {
            gui_handle.request_new_tag();
        }
        if ui.button(im_str!("New Category")) {
            gui_handle.request_new_category();
        }
        ui.separator();
        let db = gui_handle.db.read().unwrap();
        for (category_id, category) in db.categories() {
            {
                let _color = ui.push_style_color(StyleColor::Text, category.raw_color());
                if Selectable::new(&im_str!("{}", category.name))
                    .span_all_columns(false)
                    .build(ui)
                {
                    gui_handle.goto(CategoryView {
                        id: category_id,
                        edit: false,
                    });
                }
            }
            ui.same_line();
            ui.text_colored(
                [0.4, 0.4, 0.4, 1.0],
                im_str!("{}", db.tags_for_category(category_id).count()),
            );
        }
    }
}
