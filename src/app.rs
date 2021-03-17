use self::actor::AppActor;
use crate::consts::*;
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use actor::Inner;
use db::{BlobId, BlobType};
use futures_util::FutureExt;
use imgui::{
    im_str, ImStr, ImageButton, MenuItem, MouseButton, Selectable, StyleColor, TabBar, TabBarFlags,
    TabItem, TextureId, TreeNode, TreeNodeFlags, Ui, Window,
};
use piece_editor::PieceEditor;
use std::{collections::BTreeMap, ops::DerefMut, sync::Arc};
use tokio::sync::mpsc;
use winit::dpi::PhysicalSize;

pub mod actor;
pub mod piece_editor;

pub struct App {
    pub actor: Arc<AppActor>,
    pub incoming_images: mpsc::Receiver<(BlobId, RawImage, RawImage)>,
    pub images: BTreeMap<BlobId, Option<(TextureImage, TextureImage)>>,
}

impl App {
    pub fn update(&mut self, gui: &mut GuiContext) {
        let mut backend = self.actor.write();
        let Inner { .. } = backend.deref_mut();

        if let Some(Some((blob_id, raw, thumbnail))) = self.incoming_images.recv().now_or_never() {
            let image = TextureImage {
                data: gui.load(&raw),
                width: raw.width,
                height: raw.height,
            };
            let thumbnail = TextureImage {
                data: gui.load(&thumbnail),
                width: thumbnail.width,
                height: thumbnail.height,
            };

            self.images.insert(blob_id, Some((image, thumbnail)));
        }
    }

    pub fn render(&mut self, ui: &Ui<'_>, window: PhysicalSize<f32>) {
        let (mut backend, actor, images) = (self.actor.write(), &self.actor, &mut self.images);
        let Inner { backend, .. } = backend.deref_mut();

        ui.main_menu_bar(|| {
            ui.menu(im_str!("File"), true, || {
                if MenuItem::new(im_str!("New Piece")).build(ui) {
                    actor.request_new_piece();
                }
            });
            ui.menu(im_str!("Edit"), true, || {
                if MenuItem::new(im_str!("Undo"))
                    .enabled(backend.db.can_undo())
                    .build(ui)
                {
                    backend.db.undo();
                }
                if MenuItem::new(im_str!("Redo"))
                    .enabled(backend.db.can_redo())
                    .build(ui)
                {
                    backend.db.redo();
                }
            });
        });

        let color = ui.push_style_color(StyleColor::WindowBg, [0.067, 0.067, 0.067, 1.0]);

        Window::new(im_str!("Search"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .position([0.0, MAIN_MENU_BAR_OFFSET], imgui::Condition::Always)
            .size([window.width, SEARCH_BAR_HEIGHT], imgui::Condition::Always)
            .build(ui, || {});

        Window::new(im_str!("Gallery"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .position(
                [EXPLORER_WIDTH, MAIN_MENU_BAR_OFFSET + SEARCH_BAR_HEIGHT],
                imgui::Condition::Always,
            )
            .size(
                [
                    window.width - EXPLORER_WIDTH,
                    window.height - MAIN_MENU_BAR_OFFSET - SEARCH_BAR_HEIGHT,
                ],
                imgui::Condition::Always,
            )
            .build(ui, || {
                for (id, data) in backend
                    .db
                    .blobs
                    .iter()
                    .filter(|(_, blob)| blob.blob_type == BlobType::Canon)
                {
                    if let Some(requested) = images.get(&id) {
                        if let Some((_, thumbnail)) = requested {
                            imgui::ChildWindow::new(&im_str!("##{}", data.hash))
                                .size([
                                    THUMBNAIL_SIZE + IMAGE_BUFFER,
                                    THUMBNAIL_SIZE + IMAGE_BUFFER,
                                ])
                                .draw_background(false)
                                .build(ui, || {
                                    let aspect_ratio =
                                        thumbnail.width as f32 / thumbnail.height as f32;
                                    let (size, padding) = if aspect_ratio < 1.0 {
                                        (
                                            [THUMBNAIL_SIZE * aspect_ratio, THUMBNAIL_SIZE],
                                            [THUMBNAIL_SIZE * (1.0 - aspect_ratio), 0.0],
                                        )
                                    } else if aspect_ratio > 1.0 {
                                        (
                                            [THUMBNAIL_SIZE, THUMBNAIL_SIZE / aspect_ratio],
                                            [0.0, THUMBNAIL_SIZE * (1.0 - 1.0 / aspect_ratio)],
                                        )
                                    } else {
                                        ([THUMBNAIL_SIZE * aspect_ratio, THUMBNAIL_SIZE], [0.0; 2])
                                    };

                                    ui.set_cursor_pos([
                                        ui.cursor_pos()[0] + padding[0] / 2.0,
                                        ui.cursor_pos()[1] + padding[1] / 2.0,
                                    ]);

                                    imgui::ImageButton::new(thumbnail.data, size).build(ui);
                                });
                            ui.same_line(0.0);
                            if ui.content_region_avail()[0] < THUMBNAIL_SIZE + IMAGE_BUFFER {
                                ui.new_line();
                            }
                        }
                    } else {
                        images.insert(id, None);
                        actor.request_load_image(id);
                    }
                }
                ui.new_line();
            });
        color.pop(ui);

        Window::new(im_str!("Tags"))
            .movable(false)
            .resizable(false)
            .collapsible(true)
            .position(
                [0.0, MAIN_MENU_BAR_OFFSET + SEARCH_BAR_HEIGHT],
                imgui::Condition::Always,
            )
            .size(
                [
                    EXPLORER_WIDTH,
                    window.height - MAIN_MENU_BAR_OFFSET - SEARCH_BAR_HEIGHT,
                ],
                imgui::Condition::Always,
            )
            .build(ui, || {});

        ui.show_default_style_editor();
    }
}
