use super::{tag::TagView, GuiView};
use crate::app::widgets::*;
use category::EditCategoryResponse;
use db::CategoryId;
use imgui::{im_str, Selectable};
use itertools::Itertools;

#[derive(Debug)]
pub struct CategoryView {
    pub id: CategoryId,
    pub edit: bool,
}

impl GuiView for CategoryView {
    fn update(&self, gui_handle: &super::GuiHandle) {
        let db = gui_handle.db.read().unwrap();
        if !db.exists(self.id) {
            gui_handle.go_back();
        }
    }
    fn draw_main(
        &mut self,
        gui_handle: &super::GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();

        ui.columns(1, im_str!("unheader"), false);
        ui.columns(2, im_str!("tag list"), true);
        for (tag_id, tag) in db
            .tags_for_category(self.id)
            .map(|id| (id, &db[id]))
            .sorted_by_key(|(_, tag)| &tag.name)
        {
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
        }

        ui.columns(1, im_str!("untag list"), false);
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &super::GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();
        if db.exists(self.id) {
            if ui.button(&im_str!("{}", if self.edit { "View" } else { "Edit" })) {
                self.edit = !self.edit;
            }
            if !self.edit {
                category::view(self.id, &db, ui);
            } else {
                match category::edit(self.id, &db, ui) {
                    EditCategoryResponse::None => {}
                    EditCategoryResponse::Edit(edit) => {
                        gui_handle.update_category(edit);
                    }
                    EditCategoryResponse::Delete => {
                        gui_handle.delete_category(self.id);
                        gui_handle.go_back();
                    }
                }
            }
        }
    }
    fn label(&self) -> &'static str {
        "Category"
    }
}
