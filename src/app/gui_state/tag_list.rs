use chrono::Local;
use db::{BlobType, Category, Tag};
use imgui::{im_str, Selectable};

use super::{piece::PieceView, tag::TagView, GuiHandle, GuiView};
use crate::app::widgets::*;

#[derive(Debug)]
pub struct TagList;

impl GuiView for TagList {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(
        &mut self,
        gui_handle: &GuiHandle,
        gui_state: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();

        ui.columns(2, im_str!("header"), true);

        ui.text(im_str!("Name"));
        ui.next_column();
        ui.text(im_str!("Category"));

        ui.columns(1, im_str!("unheader"), false);
        ui.separator();
        ui.columns(2, im_str!("tag list"), true);
        for (tag_id, tag) in db.tags() {
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
                if Selectable::new(&im_str!("{}", db[category_id].name))
                    .span_all_columns(false)
                    .build(ui)
                {
                    todo!()
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
            //
        }
        // for i in 0..10u32 {
        //     let t = Tag {
        //         name: format!("tag_{}", i),
        //         description: format!("My test description {}", i),
        //         added: Local::today().naive_local(),
        //         links: Vec::new(),
        //     };
        //     let tg = Category {
        //         name: format!("category_{}", i),
        //         color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
        //         added: Local::today().naive_local(),
        //         ..Category::default()
        //     };

        //     tag::gallery(ui, &t, &tg);
        // }
    }
}
