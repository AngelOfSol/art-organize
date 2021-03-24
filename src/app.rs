use self::gui_state::{GuiHandle, GuiState};
use crate::layout::{Column, Dimension, LayoutIds, LayoutRectangle, Row};
use crate::{backend::actor::DbHandle, consts::*};
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use db::{BlobId, BlobType, Tag, TagCategory};
use futures_util::FutureExt;
use glam::Vec2;
use gui_state::MainWindow;
use imgui::{im_str, ChildWindow, CollapsingHeader, Key, MenuItem, MouseButton, Ui, Window};
use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, RwLock},
};
use strum::IntoEnumIterator;
use tokio::sync::mpsc;
use winit::dpi::PhysicalSize;

pub mod blob;
pub mod gallery;
pub mod gui_state;
pub mod piece;
pub mod tag;
pub mod tag_category;

pub struct App {
    pub handle: DbHandle,
    pub gui_handle: GuiHandle,
    pub gui_state: Arc<RwLock<GuiState>>,
    pub incoming_images: mpsc::UnboundedReceiver<(BlobId, RawImage, bool)>,
}

impl App {
    pub fn update(&mut self, gui: &mut GuiContext) {
        if let Some(Some((blob_id, raw, is_thumbnail))) = self.incoming_images.recv().now_or_never()
        {
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
        let gui_state = gui_state.deref_mut();

        let db = self.handle.read().unwrap();

        let db_handle = &self.handle;
        let gui_handle = &self.gui_handle;

        let layout = {
            let mut layout_data = HashMap::new();
            let layout = Column::default()
                .push(LayoutIds::MenuBar, Dimension::Pixels(20.0))
                .push(LayoutIds::SearchBar, Dimension::Pixels(40.0))
                .push(
                    Row::default()
                        .push(LayoutIds::Tags, Dimension::Pixels(240.0))
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

        if ui.is_key_pressed_no_repeat(Key::Z) && ui.io().key_ctrl && db.can_undo() {
            db_handle.undo();
        }
        if ui.is_key_pressed_no_repeat(Key::Y) && ui.io().key_ctrl && db.can_redo() {
            db_handle.redo();
        }

        if ui.is_key_pressed(Key::RightArrow) {
            gui_handle.next_item();
        }
        if ui.is_key_pressed(Key::LeftArrow) {
            gui_handle.prev_item();
        }

        ui.main_menu_bar(|| {
            ui.menu(im_str!("File"), || {
                if MenuItem::new(im_str!("New Piece")).build(ui) {
                    gui_handle.request_new_piece();
                }
            });
            ui.menu(im_str!("Edit"), || {
                if MenuItem::new(im_str!("Undo"))
                    .enabled(db.can_undo())
                    .shortcut(im_str!("Ctrl+Z"))
                    .build(ui)
                {
                    db_handle.undo();
                }
                if MenuItem::new(im_str!("Redo"))
                    .enabled(db.can_redo())
                    .shortcut(im_str!("Ctrl+Y"))
                    .build(ui)
                {
                    db_handle.redo();
                }
            });
            ui.menu(im_str!("Debug"), || {
                MenuItem::new(im_str!("Styles")).build_with_ref(ui, &mut gui_state.show_styles);

                MenuItem::new(im_str!("Metrics")).build_with_ref(ui, &mut gui_state.show_metrics);
            });
        });

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
                    .resize_buffer(true)
                    .hint(im_str!("Search"))
                    .build()
                {
                    gui_state.search.text = buf.to_string();
                };
                drop(width);
            });

        Window::new(&im_str!("{}", gui_state.main_window))
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
            .build(ui, || match &mut gui_state.main_window {
                MainWindow::Gallery => {
                    let blobs = db
                        .pieces()
                        .filter_map(|(id, _)| db.blobs_for_piece(id).next());

                    if let Some(id) = gallery::render(
                        ui,
                        blobs,
                        &gui_handle,
                        &gui_state.thumbnails,
                        |blob, ui| {
                            let piece_id = db.pieces_for_blob(blob).next().unwrap();
                            piece::view(piece_id, &db, ui);
                        },
                    ) {
                        gui_handle.request_view_piece(db.pieces_for_blob(id).next().unwrap());
                    }
                }
                MainWindow::Piece { id, focused, .. } => {
                    let content_region = ui.content_region_avail();

                    match focused {
                        Some(blob_id) => {
                            if let Some(image) = gui_state.images.get(blob_id) {
                                let zoom = (1.0
                                    / (image.width as f32 / content_region[0])
                                        .max(image.height as f32 / content_region[1]))
                                .min(1.0);

                                let size = [image.width as f32 * zoom, image.height as f32 * zoom];

                                let padded = [
                                    0.5 * (content_region[0] - size[0]) + ui.cursor_pos()[0],
                                    0.5 * (content_region[1] - size[1]) + ui.cursor_pos()[1],
                                ];

                                ui.set_cursor_pos(padded);

                                imgui::Image::new(image.data, size).build(ui);
                            } else {
                                gui_handle.request_load_image(*blob_id);
                            }
                        }
                        None => {
                            let blob_ids = db.blobs_for_piece(*id);

                            for blob_type in BlobType::iter() {
                                let _id = ui.push_id(&im_str!("{}", blob_type));
                                if CollapsingHeader::new(&im_str!("{}", blob_type))
                                    .default_open(true)
                                    .build(ui)
                                {
                                    if let Some(to_focus) = gallery::render(
                                        ui,
                                        blob_ids
                                            .clone()
                                            .filter(|blob| db[*blob].blob_type == blob_type),
                                        &gui_handle,
                                        &gui_state.thumbnails,
                                        |blob_id, ui| {
                                            blob::view(blob_id, &db, ui);
                                        },
                                    ) {
                                        *focused = Some(to_focus);
                                    }

                                    if ui.content_region_avail()[0] < THUMBNAIL_SIZE + IMAGE_BUFFER
                                    {
                                        ui.new_line();
                                    } else {
                                        ui.same_line();
                                    }
                                    ChildWindow::new(im_str!("add button"))
                                        .draw_background(false)
                                        .size([THUMBNAIL_SIZE + IMAGE_BUFFER; 2])
                                        .build(ui, || {
                                            ui.set_cursor_pos([IMAGE_BUFFER / 2.0; 2]);
                                            if ui
                                                .button_with_size(im_str!("+"), [THUMBNAIL_SIZE; 2])
                                            {
                                                // TODO move this to a gui_handle
                                                let mut recv =
                                                    db_handle.new_blobs_for_piece(*id, blob_type);
                                            };
                                        });
                                }
                            }
                        }
                    }

                    if ui.is_mouse_double_clicked(MouseButton::Right)
                        && ui.is_mouse_hovering_rect(
                            ui.window_pos(),
                            [
                                ui.window_pos()[0] + ui.window_size()[0],
                                ui.window_pos()[1] + ui.window_size()[1],
                            ],
                        )
                    {
                        match focused {
                            Some(_) => {
                                *focused = None;
                            }
                            None => {
                                gui_state.main_window = MainWindow::Gallery;
                            }
                        }
                    }
                }
            });

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
            .build(ui, || match &mut gui_state.main_window {
                MainWindow::Gallery => {
                    for i in 0..10u32 {
                        let t = Tag {
                            name: format!("tag_{}", i),
                            description: format!("My test description {}", i),
                            added: chrono::Local::now(),
                            links: Vec::new(),
                        };
                        let tg = TagCategory {
                            name: format!("category_{}", i),
                            color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
                            added: chrono::Local::now(),
                        };

                        tag::gallery(ui, &t, &tg);
                    }
                }
                MainWindow::Piece {
                    id: piece_id, edit, ..
                } => {
                    if ui.button(&im_str!("{}", if *edit { "View" } else { "Edit" })) {
                        *edit = !*edit;
                    }
                    if !*edit {
                        piece::view_with_tags(*piece_id, &db, ui);
                    } else if let Some(edit) = piece::edit(*piece_id, &db, ui) {
                        self.handle.update_piece(edit);
                    }
                }
            });
    }
}
