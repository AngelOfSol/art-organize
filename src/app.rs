use self::actor::AppActor;
use crate::consts::*;
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use actor::Inner;
use db::{BlobId, BlobType, PieceId, Tag, TagCategory};
use futures_util::FutureExt;
use gui_state::MainWindow;
use imgui::{
    im_str, ChildWindow, CollapsingHeader, ImStr, Key, MenuItem, MouseButton, Selectable,
    StyleColor, Ui, Window,
};
use std::{collections::BTreeMap, ops::DerefMut, sync::Arc};
use strum::IntoEnumIterator;
use tag::ExtraType;
use tokio::sync::mpsc;
use winit::dpi::PhysicalSize;

pub mod actor;
pub mod gui_state;
pub mod piece;
pub mod tag;

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

        if ui.is_key_pressed_no_repeat(Key::Z) && ui.io().key_ctrl && db.can_undo() {
            db.undo();
        }
        if ui.is_key_pressed_no_repeat(Key::Y) && ui.io().key_ctrl && db.can_redo() {
            db.redo();
        }

        ui.main_menu_bar(|| {
            ui.menu(im_str!("File"), || {
                if MenuItem::new(im_str!("New Piece")).build(ui) {
                    actor.request_new_piece();
                }
            });
            ui.menu(im_str!("Edit"), || {
                if MenuItem::new(im_str!("Undo"))
                    .enabled(db.can_undo())
                    .shortcut(im_str!("Ctrl+Z"))
                    .build(ui)
                {
                    db.undo();
                }
                if MenuItem::new(im_str!("Redo"))
                    .enabled(db.can_redo())
                    .shortcut(im_str!("Ctrl+Y"))
                    .build(ui)
                {
                    db.redo();
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
            .position([0.0, MAIN_MENU_BAR_OFFSET], imgui::Condition::Always)
            .size([window.width, SEARCH_BAR_HEIGHT], imgui::Condition::Always)
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
                        .map(|(_, blob)| blob)
                        .copied();

                    if let Some(id) = render_gallery(ui, blobs, &actor, images) {
                        actor.request_show_piece(id);
                    }
                }
                MainWindow::Piece { id, focused, .. } => {
                    let content_region = ui.content_region_avail();

                    match focused {
                        Some(blob_id) => {
                            if let Some(requested) = images.get(blob_id) {
                                if let Some((image, _)) = requested {
                                    let zoom = (1.0
                                        / (image.width as f32 / content_region[0])
                                            .max(image.height as f32 / content_region[1]))
                                    .min(1.0);

                                    let size =
                                        [image.width as f32 * zoom, image.height as f32 * zoom];

                                    let padded = [
                                        0.5 * (content_region[0] - size[0]) + ui.cursor_pos()[0],
                                        0.5 * (content_region[1] - size[1]) + ui.cursor_pos()[1],
                                    ];

                                    ui.set_cursor_pos(padded);

                                    imgui::Image::new(image.data, size).build(ui);
                                }
                            } else if !images.contains_key(blob_id) {
                                images.insert(*blob_id, None);
                                actor.request_load_image(*blob_id);
                            }
                        }
                        None => {
                            let blob_ids = db
                                .media
                                .iter()
                                .filter(|(piece, _)| piece == id)
                                .map(|(_, blob)| blob)
                                .copied();
                            for blob_type in BlobType::iter() {
                                let _id = ui.push_id(&im_str!("{}", blob_type));
                                if CollapsingHeader::new(&im_str!("{}", blob_type))
                                    .default_open(true)
                                    .build(ui)
                                {
                                    ui.group(|| {
                                        if let Some(to_focus) = render_gallery(
                                            ui,
                                            blob_ids.clone().filter(|blob| {
                                                db.blobs[*blob].blob_type == blob_type
                                            }),
                                            &actor,
                                            images,
                                        ) {
                                            *focused = Some(to_focus);
                                        }

                                        if ui.content_region_avail()[0]
                                            < THUMBNAIL_SIZE + IMAGE_BUFFER
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
                                                ui.button_with_size(
                                                    im_str!("+"),
                                                    [THUMBNAIL_SIZE; 2],
                                                );
                                            });
                                    });
                                    if ui.is_mouse_hovering_rect(
                                        ui.item_rect_min(),
                                        [
                                            ui.cursor_screen_pos()[0]
                                                + ui.content_region_avail()[0],
                                            ui.item_rect_max()[1],
                                        ],
                                    ) {
                                        // dbg!("hoverign");
                                        //
                                    }
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
                    // let blob_id = db
                    //     .media
                    //     .iter()
                    //     .find(|(piece, _)| piece == id)
                    //     .map(|(_, blob)| blob);

                    // if let Some(requested) = blob_id.and_then(|blob_id| images.get(blob_id)) {
                    //     if let Some((image, _)) = requested {
                    //         let zoom = (1.0
                    //             / (image.width as f32 / content_region[0])
                    //                 .max(image.height as f32 / content_region[1]))
                    //         .min(1.0);

                    //         let size = [image.width as f32 * zoom, image.height as f32 * zoom];

                    //         let padded = [
                    //             0.5 * (content_region[0] - size[0]) + ui.cursor_pos()[0],
                    //             0.5 * (content_region[1] - size[1]) + ui.cursor_pos()[1],
                    //         ];

                    //         ui.set_cursor_pos(padded);

                    //         imgui::Image::new(image.data, size).build(ui);
                    //     }
                    // } else if let Some(blob_id) = blob_id {
                    //     images.insert(*blob_id, None);
                    //     actor.request_load_image(*blob_id);
                    // }
                }
            });

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
                        let raw_color = [
                            tg.color[0] as f32 / 255.0,
                            tg.color[1] as f32 / 255.0,
                            tg.color[2] as f32 / 255.0,
                            tg.color[3] as f32 / 255.0,
                        ];

                        tag::tag_old(ui, &im_str!("{}", t.name), raw_color, ExtraType::Gallery);
                    }
                }
                MainWindow::Piece { id, edit, .. } => {
                    if ui.button(&im_str!("{}", if *edit { "View" } else { "Edit" })) {
                        *edit = !*edit;
                    }
                    if !*edit {
                        let piece = &db.pieces[*id];

                        ui.text_wrapped(&im_str!("Name: {}", piece.name));
                        ui.text_wrapped(&im_str!("Source Type: {}", piece.source_type));
                        ui.text_wrapped(&im_str!("Media Type: {}", piece.media_type));
                        ui.text(im_str!(
                            "Date Added: {}",
                            piece.added.format("%-m/%-d/%-Y %-H:%-M %P")
                        ));
                        if let Some(price) = piece.base_price {
                            ui.text(im_str!("Price: ${}", price));
                        }
                        if let Some(price) = piece.tip_price {
                            ui.text(im_str!("Tipped: ${}", price));
                        }

                        ui.separator();

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

                            tag_category(ui, &im_str!("{}", tg.name), raw_color);
                            ui.indent();
                            for j in 0..2 {
                                let t = Tag {
                                    name: format!("tag_{}", j),
                                    description: format!("My test description {}", j),
                                    added: chrono::Local::now(),
                                    links: Vec::new(),
                                };
                                let label = im_str!("{}", t.name);
                                tag::tag_old(ui, &label, raw_color, ExtraType::View);
                            }
                            ui.unindent();
                        }
                    } else {
                        piece::edit(*id, db, ui);
                    }
                }
            });
    }
}

fn tag_category(ui: &Ui, label: &ImStr, raw_color: [f32; 4]) -> bool {
    let ret = Selectable::new(im_str!("?"))
        .size([ui.text_line_height_with_spacing(); 2])
        .build(ui);
    ui.same_line();
    ui.text_colored(raw_color, label);
    ret
}

fn render_gallery<I: Iterator<Item = BlobId>>(
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

    for blob in blobs {
        if let Some(requested) = images.get(&blob) {
            if let Some((image, thumbnail)) = requested {
                imgui::ChildWindow::new(&im_str!("##{:?}", blob))
                    .size([THUMBNAIL_SIZE + IMAGE_BUFFER, THUMBNAIL_SIZE + IMAGE_BUFFER])
                    .draw_background(false)
                    .build(ui, || {
                        let (size, padding) = rescale(thumbnail, [THUMBNAIL_SIZE; 2]);
                        ui.set_cursor_pos([
                            ui.cursor_pos()[0] + padding[0] / 2.0 + IMAGE_BUFFER / 2.0,
                            ui.cursor_pos()[1] + padding[1] / 2.0 + IMAGE_BUFFER / 2.0,
                        ]);

                        if imgui::ImageButton::new(thumbnail.data, size).build(ui) {
                            ret = Some(blob);
                        }

                        if ui.is_item_hovered() {
                            ui.tooltip(|| {
                                ui.text(im_str!("Piece Name Here"));
                            });
                        }
                    });

                ui.same_line();
                if ui.content_region_avail()[0] < THUMBNAIL_SIZE + IMAGE_BUFFER {
                    ui.new_line();
                }
            }
        } else {
            images.insert(blob, None);
            actor.request_load_image(blob);
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
