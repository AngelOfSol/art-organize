use std::borrow::BorrowMut;

use imgui::im_str;

use super::{gallery::Gallery, help::Help, tag_list::TagList, GuiHandle, GuiView};

#[derive(Debug)]
pub struct Home;

impl GuiView for Home {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(&mut self, gui_handle: &GuiHandle, _: &super::InnerGuiState, ui: &imgui::Ui<'_>) {
        ui.text(im_str!("Home Screen"));
        ui.separator();
        if ui.button(im_str!("Gallery")) {
            gui_handle.goto(Gallery);
        }
        if ui.button(im_str!("Tags")) {
            gui_handle.goto(TagList);
        }
        if ui.button(im_str!("Help")) {
            gui_handle.goto(Help);
        }
    }

    fn draw_explorer(&mut self, _: &GuiHandle, _: &super::InnerGuiState, _: &imgui::Ui<'_>) {}
}
