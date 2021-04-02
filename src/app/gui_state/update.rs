use imgui::{im_str, PopupModal};

use super::{GuiHandle, GuiView};

#[derive(Debug)]
pub struct CheckForUpdates {
    pub version: Option<String>,
}

impl GuiView for CheckForUpdates {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(&mut self, gui_handle: &GuiHandle, _: &super::InnerGuiState, ui: &imgui::Ui<'_>) {
        let mut showing_popup = false;
        PopupModal::new(im_str!("Check for Updates"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .always_auto_resize(true)
            .build(ui, || {
                if let Some(version) = &self.version {
                    ui.text(im_str!("New version available: {}", version));
                    ui.text(im_str!("Updating will close the program."));
                    ui.separator();
                    if ui.button(im_str!("Yes, update")) {
                        ui.close_current_popup();
                        let gui_handle = (*gui_handle).clone();
                        tokio::task::spawn_blocking(move || match crate::updater::update_app() {
                            Ok(value) => match value {
                                self_update::Status::UpToDate(_) => gui_handle.go_back(),
                                self_update::Status::Updated(_) => std::process::exit(0),
                            },
                            Err(e) => {
                                gui_handle.go_back();
                                dbg!(e);
                            }
                        });
                    }
                    ui.same_line();
                    if ui.button(im_str!("Cancel")) {
                        ui.close_current_popup();
                        gui_handle.go_back();
                    }
                } else {
                    ui.text(im_str!("No new versions available!"));
                    ui.separator();
                    if ui.button(im_str!("Ok")) {
                        ui.close_current_popup();
                        gui_handle.go_back();
                    }
                }
                showing_popup = true;
            });
        if !showing_popup {
            ui.open_popup(im_str!("Check for Updates"));
        }
    }

    fn draw_explorer(&mut self, _: &GuiHandle, _: &super::InnerGuiState, _: &imgui::Ui<'_>) {}
}
