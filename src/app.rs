use self::actor::AppActor;
use crate::consts::*;
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use actor::Inner;
use db::{Blob, BlobId, Tag, TagCategory};
use futures_util::FutureExt;
use gui_state::MainWindow;
use imgui::{
    im_str, ChildWindow, ImStr, MenuItem, MouseButton, Selectable, StyleColor, Ui, Window,
};
use std::{collections::BTreeMap, ops::DerefMut, sync::Arc};
use tokio::sync::mpsc;
use winit::dpi::PhysicalSize;

pub mod actor;
pub mod gui_state;
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
        let Inner { db, gui_state, .. } = backend.deref_mut();

        ui.main_menu_bar(|| {
            ui.menu(im_str!("File"), || {
                if MenuItem::new(im_str!("New Piece")).build(ui) {
                    actor.request_new_piece();
                }
            });
            ui.menu(im_str!("Edit"), || {
                if MenuItem::new(im_str!("Undo"))
                    .enabled(db.can_undo())
                    .build(ui)
                {
                    db.undo();
                }
                if MenuItem::new(im_str!("Redo"))
                    .enabled(db.can_redo())
                    .build(ui)
                {
                    db.redo();
                }
            });
        });

        Window::new(im_str!("Search"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .position([0.0, MAIN_MENU_BAR_OFFSET], imgui::Condition::Always)
            .size([window.width, SEARCH_BAR_HEIGHT], imgui::Condition::Always)
            .build(ui, || {});

        Window::new(&im_str!("{}", gui_state.main_window))
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
            .build(ui, || match &mut gui_state.main_window {
                MainWindow::Gallery => {
                    let blobs = db
                        .pieces
                        .keys()
                        .filter_map(|id| db.media.iter().find(|(piece, _)| piece == &id))
                        .map(|(_, blob)| (*blob, &db.blobs[*blob]));

                    if let Some(id) = render_gallery(ui, blobs, &actor, images) {
                        actor.request_show_blob(id);
                    }
                }
                MainWindow::Blob { id } => {
                    let content_region = ui.content_region_avail();

                    if let Some(requested) = images.get(&id) {
                        if let Some((image, _)) = requested {
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
                        }
                    } else {
                        images.insert(*id, None);
                        actor.request_load_image(*id);
                    }
                }
            });

        let button_size = [ui.text_line_height_with_spacing(); 2];
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
            .build(ui, || match gui_state.main_window {
                MainWindow::Gallery => {
                    for i in 0..10u32 {
                        let t = Tag {
                            name: format!("tag_{}", i),
                            description: format!("My test description {}", i),
                            added: chrono::Local::now(),
                        };
                        let tg = TagCategory {
                            name: format!("category_{}", i),
                            color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
                            added: chrono::Local::now(),
                        };
                        let raw_color = [
                            tg.color[0] as f32 / 255.0,
                            tg.color[1] as f32 / 255.0,
                            tg.color[2] as f32 / 255.0,
                            tg.color[3] as f32 / 255.0,
                        ];

                        tag(ui, &im_str!("{}", t.name), button_size, raw_color, true);
                    }
                }
                MainWindow::Blob { .. } => {
                    for i in 0..10u32 {
                        let tg = TagCategory {
                            name: format!("category_{}", i),
                            color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
                            added: chrono::Local::now(),
                        };
                        let raw_color = [
                            tg.color[0] as f32 / 255.0,
                            tg.color[1] as f32 / 255.0,
                            tg.color[2] as f32 / 255.0,
                            tg.color[3] as f32 / 255.0,
                        ];

                        tag_category(ui, &im_str!("{}", tg.name), button_size, raw_color);
                        ui.indent();
                        for j in 0..2 {
                            let t = Tag {
                                name: format!("tag_{}", j),
                                description: format!("My test description {}", j),
                                added: chrono::Local::now(),
                            };
                            let label = im_str!("{}", t.name);
                            tag(ui, &label, button_size, raw_color, false);
                        }
                        ui.unindent();
                    }
                }
            });

        ui.show_default_style_editor();
    }
}

pub enum TagResponse {
    None,
    Info,
    Add,
    AddNegated,
    ReplaceSearch,
}

fn tag(
    ui: &Ui,
    label: &ImStr,
    button_size: [f32; 2],
    raw_color: [f32; 4],
    show_extras: bool,
) -> TagResponse {
    let _id = ui.push_id(label);

    if Selectable::new(im_str!("?")).size(button_size).build(ui) {
        return TagResponse::Info;
    }
    ui.same_line();

    if show_extras {
        if Selectable::new(im_str!("+")).size(button_size).build(ui) {
            return TagResponse::Info;
        }
        ui.same_line();
        if Selectable::new(im_str!("-")).size(button_size).build(ui) {
            return TagResponse::Info;
        }

        ui.same_line();
    }

    let _color = ui.push_style_color(StyleColor::Text, raw_color);
    if Selectable::new(&label).build(ui) {
        TagResponse::ReplaceSearch
    } else {
        TagResponse::None
    }
}

fn tag_category(ui: &Ui, label: &ImStr, button_size: [f32; 2], raw_color: [f32; 4]) -> bool {
    let ret = Selectable::new(im_str!("?")).size(button_size).build(ui);
    ui.same_line();
    ui.text_colored(raw_color, label);
    ret
}

fn render_gallery<'a, I: Iterator<Item = (BlobId, &'a Blob)>>(
    ui: &Ui,
    blobs: I,
    actor: &Arc<AppActor>,
    images: &mut BTreeMap<BlobId, Option<(TextureImage, TextureImage)>>,
) -> Option<BlobId> {
    let mut ret = None;
    let content_region = [
        ui.window_content_region_max()[0] / 2.0,
        ui.window_content_region_max()[1] / 2.0,
    ];

    for (id, data) in blobs {
        if let Some(requested) = images.get(&id) {
            if let Some((image, thumbnail)) = requested {
                imgui::ChildWindow::new(&im_str!("##{}", data.hash))
                    .size([THUMBNAIL_SIZE + IMAGE_BUFFER, THUMBNAIL_SIZE + IMAGE_BUFFER])
                    .draw_background(false)
                    .build(ui, || {
                        let (size, padding) = rescale(thumbnail, [THUMBNAIL_SIZE; 2]);
                        ui.set_cursor_pos([
                            ui.cursor_pos()[0] + padding[0] / 2.0,
                            ui.cursor_pos()[1] + padding[1] / 2.0,
                        ]);

                        if imgui::ImageButton::new(thumbnail.data, size).build(ui) {
                            ret = Some(id);
                        }

                        if ui.is_item_hovered() {
                            ui.tooltip(|| {
                                let (size, _) = rescale(image, content_region);

                                imgui::Image::new(image.data, size).build(ui);
                            });
                        }
                    });

                ui.same_line();
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

    ret
}
fn rescale(image: &TextureImage, max_size: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    rescale_with_zoom(image, max_size, 1.0)
}
fn rescale_with_zoom(image: &TextureImage, max_size: [f32; 2], zoom: f32) -> ([f32; 2], [f32; 2]) {
    let size = [image.width as f32 * zoom, image.height as f32 * zoom];
    let aspect_ratio = size[0] / size[1];
    let new_aspect_ratio = max_size[0] / max_size[1];

    let size = if size[0] <= max_size[0] && size[1] <= max_size[1] {
        size
    } else {
        let use_width = aspect_ratio >= new_aspect_ratio;

        if use_width {
            [max_size[0], size[1] * max_size[0] / size[0]]
        } else {
            [size[0] * max_size[1] / size[1], max_size[1]]
        }
    };

    (size, [max_size[0] - size[0], max_size[1] - size[1]])
}
