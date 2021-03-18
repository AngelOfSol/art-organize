use self::actor::AppActor;
use crate::consts::*;
use crate::{
    gui::GuiContext,
    raw_image::{RawImage, TextureImage},
};
use actor::Inner;
use db::{Blob, BlobId, PieceId, Tag, TagCategory};
use futures_util::FutureExt;
use gui_state::MainWindow;
use imgui::{
    im_str, ChildWindow, ImStr, MenuItem, MouseButton, Selectable, StyleColor, Ui, Window,
};
use imgui_sys::{ImGuiCond_Always, ImVec2};
use std::{collections::BTreeMap, ops::DerefMut, slice::SliceIndex, sync::Arc};
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
                    .callback_completion(true)
                    .build()
                {
                    gui_state.search.text = buf.to_string();
                };
                drop(width);
                if ui.is_item_focused()
                    && !gui_state.search.text.ends_with(r"\s")
                    && !gui_state.search.text.is_empty()
                {
                    unsafe {
                        imgui_sys::igSetNextWindowPos(
                            ui.cursor_screen_pos().into(),
                            std::convert::TryInto::try_into(ImGuiCond_Always).unwrap(),
                            ImVec2::new(0.0, 0.0),
                        );
                    }
                    ui.tooltip(|| {
                        if gui_state.search.auto_complete.is_empty() {
                            gui_state.search.auto_complete = vec![
                                "tag1".to_string(),
                                "artist:tag1".to_string(),
                                "tag2".to_string(),
                                "tag3".to_string(),
                            ];
                        }
                        if ui.is_key_pressed(imgui::Key::DownArrow) {
                            match &mut gui_state.search.selected {
                                Some(data) => {
                                    *data += 1;
                                    if gui_state.search.auto_complete.get(*data).is_none() {
                                        gui_state.search.selected = None;
                                    }
                                }
                                None => {
                                    if !gui_state.search.auto_complete.is_empty() {
                                        gui_state.search.selected = Some(0);
                                    }
                                }
                            }
                        }

                        if ui.is_key_pressed(imgui::Key::UpArrow) {
                            match &mut gui_state.search.selected {
                                Some(data) => {
                                    gui_state.search.selected = data.checked_sub(1);
                                }
                                None => {
                                    if !gui_state.search.auto_complete.is_empty() {
                                        gui_state.search.selected =
                                            Some(gui_state.search.auto_complete.len() - 1);
                                    }
                                }
                            }
                        }

                        if ui.is_key_pressed(imgui::Key::Tab) {
                            if let Some(value) = dbg!(gui_state.search.selected)
                                .and_then(|idx| dbg!(gui_state.search.auto_complete.get(idx)))
                                .cloned()
                            {
                                gui_state.search.text.push_str(&value);
                                dbg!(&gui_state.search.text);
                            }
                        }

                        for (idx, tag) in gui_state.search.auto_complete.iter().enumerate() {
                            Selectable::new(&im_str!("{}", tag))
                                .selected(Some(idx) == gui_state.search.selected)
                                .build(ui);
                        }
                    });
                }
            });
        let gui_state = &*gui_state;

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
            .build(ui, || match &gui_state.main_window {
                MainWindow::Gallery => {
                    let blobs = db
                        .pieces
                        .keys()
                        .filter_map(|id| db.media.iter().find(|(piece, _)| piece == &id))
                        .copied();

                    if let Some(id) = render_gallery(ui, blobs, &actor, images) {
                        actor.request_show_piece(id);
                    }
                }
                MainWindow::Piece { id: blob_id } => {
                    let content_region = ui.content_region_avail();

                    let blob_id = db
                        .media
                        .iter()
                        .find(|(piece, _)| piece == blob_id)
                        .map(|(_, blob)| blob);

                    if let Some(requested) = blob_id.and_then(|id| images.get(id)) {
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
                    } else if let Some(blob_id) = blob_id {
                        images.insert(*blob_id, None);
                        actor.request_load_image(*blob_id);
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

                        tag(ui, &im_str!("{}", t.name), button_size, raw_color, true);
                    }
                }
                MainWindow::Piece { id } => {
                    let piece = &db.pieces[id];

                    ui.text(im_str!("Name: {}", piece.name));
                    ui.text(im_str!("Added: {}", piece.added));
                    ui.text(im_str!("Source Type: {}", piece.source_type));
                    ui.text(im_str!("Media Type: {}", piece.media_type));
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

                        tag_category(ui, &im_str!("{}", tg.name), button_size, raw_color);
                        ui.indent();
                        for j in 0..2 {
                            let t = Tag {
                                name: format!("tag_{}", j),
                                description: format!("My test description {}", j),
                                added: chrono::Local::now(),
                                links: Vec::new(),
                            };
                            let label = im_str!("{}", t.name);
                            tag(ui, &label, button_size, raw_color, false);
                        }
                        ui.unindent();
                    }
                }
            });
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

fn render_gallery<I: Iterator<Item = (PieceId, BlobId)>>(
    ui: &Ui,
    blobs: I,
    actor: &Arc<AppActor>,
    images: &mut BTreeMap<BlobId, Option<(TextureImage, TextureImage)>>,
) -> Option<PieceId> {
    let mut ret = None;
    let content_region = [
        ui.window_content_region_max()[0] / 2.0,
        ui.window_content_region_max()[1] / 2.0,
    ];

    for (piece, blob) in blobs {
        if let Some(requested) = images.get(&blob) {
            if let Some((image, thumbnail)) = requested {
                imgui::ChildWindow::new(&im_str!("##{:?}", piece))
                    .size([THUMBNAIL_SIZE + IMAGE_BUFFER, THUMBNAIL_SIZE + IMAGE_BUFFER])
                    .draw_background(false)
                    .build(ui, || {
                        let (size, padding) = rescale(thumbnail, [THUMBNAIL_SIZE; 2]);
                        ui.set_cursor_pos([
                            ui.cursor_pos()[0] + padding[0] / 2.0,
                            ui.cursor_pos()[1] + padding[1] / 2.0,
                        ]);

                        if imgui::ImageButton::new(thumbnail.data, size).build(ui) {
                            ret = Some(piece);
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
