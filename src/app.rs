use self::gui_state::{GuiHandle, GuiState};
use crate::layout::{Column, Dimension, LayoutIds, LayoutRectangle, Row};
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use db::BlobId;
use glam::Vec2;
use imgui::{im_str, Key, MenuItem, MouseButton, Ui, Window};
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
                let width = ui.push_item_width(-1.0);
                let mut buf = gui_state.search.text.clone().into();
                if ui
                    .input_text(im_str!("##Search Input"), &mut buf)
                    .callback_always(true)
                    .hint(im_str!("Search"))
                    .build()
                {
                    gui_state.search.text = buf.to_string();
                };
                drop(width);
            });
        Window::new(im_str!("Main"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .position(
                layout[&LayoutIds::Main].position.into(),
                imgui::Condition::Always,
            )
            .size(
                layout[&LayoutIds::Main].size.into(),
                imgui::Condition::Always,
            )
            .build(ui, || gui_state.render_main(&self.gui_handle, ui));

        Window::new(im_str!("Tags"))
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
