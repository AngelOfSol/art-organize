use self::gui_state::{GuiHandle, GuiState};
use crate::{
    app::gui_state::update::CheckForUpdates,
    layout::{Column, Dimension, LayoutIds, LayoutRectangle, Row},
};
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use db::BlobId;
use glam::Vec2;
use gui_state::help::Help;
use imgui::{im_str, Key, MenuItem, MouseButton, PopupModal, Ui, Window};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, RwLock},
};
use winit::dpi::PhysicalSize;

pub mod gui_state;
pub mod widgets;

use widgets::*;

pub struct App {
    pub gui_handle: GuiHandle,
    pub gui_state: Arc<RwLock<GuiState>>,
    pub incoming_images: mpsc::Receiver<(BlobId, RawImage, bool)>,
}

enum Popup {
    CleanBlobs,
}

impl App {
    pub fn update(&mut self, gui: &mut GuiContext) {
        if let Ok((blob_id, raw, is_thumbnail)) = self.incoming_images.try_recv() {
            let image = TextureImage {
                data: gui.load(&raw),
                width: raw.width,
                height: raw.height,
                hash: raw.hash,
            };

            self.gui_handle.forward_image(blob_id, image, is_thumbnail);
        }
        {
            let mut gui_state = self.gui_state.write().unwrap();
            let db = self.gui_handle.read().unwrap();
            let invalid = gui_state
                .images
                .keys()
                .filter(|id| !db.exists(**id))
                .copied()
                .collect::<Vec<_>>();
            for key in invalid {
                gui_state.invalidate(&key);
            }
        }
    }

    pub fn render(&mut self, ui: &Ui<'_>, window: PhysicalSize<f32>) {
        let mut gui_state = self.gui_state.write().unwrap();

        gui_state.update(&self.gui_handle);

        let layout = {
            let mut layout_data = HashMap::new();
            let layout = Column::default()
                .push(LayoutIds::MenuBar, Dimension::Pixels(20.0))
                .push(LayoutIds::SearchBar, Dimension::Pixels(40.0))
                .push(
                    Row::default()
                        .push(LayoutIds::Tags, Dimension::Pixels(300.0))
                        .push(LayoutIds::Main, Dimension::Flex(1.0)),
                    Dimension::Flex(1.0),
                );

            layout.layout(
                LayoutRectangle {
                    position: Vec2::ZERO,
                    size: Vec2::new(window.width, window.height),
                },
                &mut layout_data,
            );

            layout_data
        };
        let mut trigger = None;
        {
            let db = self.gui_handle.read().unwrap();

            if ui.is_key_pressed_no_repeat(Key::Z) && ui.io().key_ctrl && db.can_undo() {
                self.gui_handle.undo();
            }
            if ui.is_key_pressed_no_repeat(Key::Y) && ui.io().key_ctrl && db.can_redo() {
                self.gui_handle.redo();
            }
            ui.main_menu_bar(|| {
                ui.menu(im_str!("File"), || {
                    if MenuItem::new(im_str!("New Database")).build(ui) {
                        self.gui_handle.new_db();
                    }
                    if MenuItem::new(im_str!("Load Database")).build(ui) {
                        self.gui_handle.load_db();
                    }
                    ui.separator();
                    if MenuItem::new(im_str!("Clean Blobs")).build(ui) {
                        trigger = Some(Popup::CleanBlobs);
                    }
                });
                ui.menu(im_str!("Data"), || {
                    if MenuItem::new(im_str!("New Piece")).build(ui) {
                        self.gui_handle.request_new_piece();
                    }
                });

                ui.menu(im_str!("Edit"), || {
                    if MenuItem::new(im_str!("Undo"))
                        .enabled(db.can_undo())
                        .shortcut(im_str!("Ctrl+Z"))
                        .build(ui)
                    {
                        self.gui_handle.undo();
                    }
                    if MenuItem::new(im_str!("Redo"))
                        .enabled(db.can_redo())
                        .shortcut(im_str!("Ctrl+Y"))
                        .build(ui)
                    {
                        self.gui_handle.redo();
                    }
                });
                ui.menu(im_str!("Help"), || {
                    if MenuItem::new(im_str!("Help")).build(ui) {
                        self.gui_handle.goto(Help)
                    }
                    ui.separator();
                    if MenuItem::new(im_str!("Check for Updates")).build(ui) {
                        let gui_handle = self.gui_handle.clone();
                        tokio::task::spawn_blocking(move || {
                            if let Ok(version) = crate::updater::check_for_new_releases() {
                                gui_handle.goto(CheckForUpdates { version });
                            }
                        });
                    }
                });
                ui.menu(im_str!("Debug"), || {
                    MenuItem::new(im_str!("Styles")).build_with_ref(ui, &mut gui_state.show_styles);

                    MenuItem::new(im_str!("Metrics"))
                        .build_with_ref(ui, &mut gui_state.show_metrics);
                });
            });
        }

        if gui_state.show_styles {
            ui.show_default_style_editor();
        }
        if gui_state.show_metrics {
            ui.show_metrics_window(&mut gui_state.show_metrics);
        }

        Window::new(im_str!("Search"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .position(
                layout[&LayoutIds::SearchBar].position.into(),
                imgui::Condition::Always,
            )
            .size(
                layout[&LayoutIds::SearchBar].size.into(),
                imgui::Condition::Always,
            )
            .build(ui, || {
                let _width = ui.push_item_width(-1.0);
                let mut buf = gui_state.search.text.clone().into();
                if ui
                    .input_text(im_str!("##Search Input"), &mut buf)
                    .resize_buffer(true)
                    .hint(im_str!("Search"))
                    .build()
                {
                    gui_state.search.text = buf.to_string();
                };
                if ui.is_item_hovered() {
                    ui.tooltip_text(im_str!("Currently unimplemented."));
                }
            });
        Window::new(im_str!("Main"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .title_bar(false)
            .position(
                layout[&LayoutIds::Main].position.into(),
                imgui::Condition::Always,
            )
            .size(
                layout[&LayoutIds::Main].size.into(),
                imgui::Condition::Always,
            )
            .build(ui, || {
                gui_state.render_main(&self.gui_handle, ui);
            });

        if let Some(popup) = trigger {
            match popup {
                Popup::CleanBlobs => ui.open_popup(im_str!("Clean Database Directory")),
            }
        }

        PopupModal::new(im_str!("Clean Database Directory"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .always_auto_resize(true)
            .build(ui, || {
                ui.text(im_str!(
                    "This will move all deleted blobs to the system recycle bin."
                ));
                ui.text(im_str!(
                    "If you undo past this point, you will need to manually"
                ));
                ui.text(im_str!("restore the old files in the database directory."));
                ui.separator();
                if ui.button(im_str!("Yes, clean deleted blobs")) {
                    self.gui_handle.clean_blobs();
                    ui.close_current_popup();
                }
                ui.same_line();
                if ui.button(im_str!("Cancel")) {
                    ui.close_current_popup();
                }
            });

        Window::new(im_str!("Explorer"))
            .movable(false)
            .resizable(false)
            .collapsible(true)
            .position(
                layout[&LayoutIds::Tags].position.into(),
                imgui::Condition::Always,
            )
            .size(
                layout[&LayoutIds::Tags].size.into(),
                imgui::Condition::Always,
            )
            .build(ui, || gui_state.render_explorer(&self.gui_handle, ui));

        if ui.is_mouse_double_clicked(MouseButton::Right) {
            self.gui_handle.go_back();
        }
    }
}
