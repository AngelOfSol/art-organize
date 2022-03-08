use db::{BlobId, PieceId, TagId};
use egui::{
    Button, CentralPanel, Color32, Frame, ImageButton, Label, Layout, PointerButton, RichText,
    ScrollArea, Sense, SidePanel, TopBottomPanel, Vec2, Window,
};
use egui_demo_lib::easy_mark::easy_mark;
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::texture_storage::{ImageData, ImageRequestType, ImageStatus},
};

pub mod texture_storage;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Gallery,
    Piece((PieceId, Option<BlobId>)),
    Fullscreen(BlobId),
}

pub struct Frontend {
    history: Vec<Mode>,
    pub image_data: ImageData,
}

impl Frontend {
    pub fn new(image_data: ImageData) -> Self {
        Self {
            history: vec![Mode::Gallery],
            image_data,
        }
    }
}

impl Frontend {
    pub fn update(&mut self, db: &mut DbBackend, ctx: &egui::CtxRef) {
        TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let mut pop_to = None;
                for (idx, mode) in self.history.iter().enumerate() {
                    let last = idx == self.history.len() - 1;
                    let response = match mode {
                        Mode::Gallery => ui.selectable_label(false, "Gallery"),
                        Mode::Piece(_) => ui.selectable_label(false, "Piece"),
                        Mode::Fullscreen(_) => ui.selectable_label(false, "View"),
                    };
                    if response.clicked() {
                        pop_to = Some(idx);
                    }

                    if !last {
                        ui.separator();
                    }
                }
                if let Some(pop_to) = pop_to {
                    self.history.resize_with(pop_to + 1, || unreachable!());
                }
            });
        });

        match *self.history.last().unwrap() {
            Mode::Gallery | Mode::Fullscreen(_) => (),
            Mode::Piece((piece_id, current_blob_id)) => {
                SidePanel::left("information")
                    .resizable(false)
                    .show(ctx, |ui| {
                        let piece = &db[piece_id];
                        ui.label(format!(
                            "External ID: {}",
                            piece.external_id.as_deref().unwrap_or("<none>")
                        ));
                        ui.label(format!("Added: {}", piece.added));
                        if let Some(price) = piece.base_price {
                            ui.label(format!("Price: ${}", price));
                        }
                        if let Some(price) = piece.tip_price {
                            ui.label(format!("Tip: ${}", price));
                        }
                        if piece.description.trim() != "" {
                            ui.separator();
                            easy_mark(ui, &piece.description);
                        }
                        ui.separator();

                        for category_id in db
                            .tags_for_piece(piece_id)
                            .flat_map(|tag| db.category_for_tag(tag))
                            .sorted_by_key(|category_id| &db[category_id].name)
                            .dedup()
                        {
                            ui.label(&db[category_id].name);

                            ui.indent("category_indent", |ui| {
                                for tag_id in db
                                    .tags_for_piece(piece_id)
                                    .filter(|tag_id| {
                                        db.category_for_tag(*tag_id) == Some(category_id)
                                    })
                                    .sorted_by_key(|tag_id| &db[tag_id].name)
                                {
                                    tag_label(ui, db, tag_id);
                                }
                            });
                        }

                        for tag_id in db
                            .tags_for_piece(piece_id)
                            .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
                            .sorted_by_key(|tag_id| &db[tag_id].name)
                        {
                            tag_label(ui, db, tag_id);
                        }
                    });
                TopBottomPanel::bottom("image_list")
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_min_height(276.0);
                        ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for blob_id in db
                                    .blobs_for_piece(piece_id)
                                    .sorted_by_key(|item| (db[item].blob_type, db[item].added))
                                {
                                    match self.image_data.thumbnail_for(blob_id, db) {
                                        ImageStatus::Available(texture) => {
                                            let response = ui.add(
                                                ImageButton::new(
                                                    texture.id,
                                                    texture.with_height(256.0),
                                                )
                                                .selected(current_blob_id == Some(blob_id)),
                                            );

                                            if response.clicked() {
                                                self.history.pop();

                                                self.history
                                                    .push(Mode::Piece((piece_id, Some(blob_id))));
                                            }
                                        }
                                        ImageStatus::Unavailable => {
                                            ui.add_sized(
                                                [256.0, 256.0],
                                                Button::new(&db[blob_id].file_name),
                                            );
                                        }
                                    }
                                }
                            });
                        });
                    });
            }
        }
        CentralPanel::default().show(ctx, |ui| {
            match *self.history.last().unwrap() {
                Mode::Gallery => self.gallery_view(ui, db),
                Mode::Piece(state) => self.piece_view(ui, db, state),
                Mode::Fullscreen(blob_id) => {
                    if let ImageStatus::Available(texture) = self.image_data.image_for(blob_id, db)
                    {
                        ui.centered_and_justified(|ui| {
                            ui.image(texture.id, texture.scaled(ui.available_size().into()));
                        });
                    }
                }
            };
        });
    }

    fn piece_view(
        &mut self,
        ui: &mut egui::Ui,
        db: &mut DbBackend,
        (_, blob_id): (PieceId, Option<BlobId>),
    ) {
        if let Some(blob_id) = blob_id {
            if let ImageStatus::Available(texture) = self.image_data.image_for(blob_id, db) {
                ui.centered_and_justified(|ui| {
                    if ui
                        .add(
                            ImageButton::new(
                                texture.id,
                                texture.scaled(ui.available_size().into()),
                            )
                            .selected(false)
                            .frame(false),
                        )
                        .double_clicked()
                    {
                        self.history.push(Mode::Fullscreen(blob_id));
                    }
                });
            }
        } else {
            ui.label("No Image");
        }
    }

    fn gallery_view(&mut self, ui: &mut egui::Ui, db: &mut DbBackend) {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (piece_id, _) in db.pieces().sorted_by_key(|(_, item)| item.added).rev() {
                        let blob_id = db
                            .blobs_for_piece(piece_id)
                            .sorted_by_key(|item| (db[item].blob_type, db[item].added))
                            .next();
                        if let Some(blob_id) = blob_id {
                            if let ImageStatus::Available(image) =
                                self.image_data.thumbnail_for(blob_id, db)
                            {
                                let response =
                                    ui.add(ImageButton::new(image.id, image.with_height(256.0)));
                                if response.clicked_by(PointerButton::Primary) {
                                    self.history.push(Mode::Piece((piece_id, Some(blob_id))));
                                }
                            }
                        }
                    }
                });
            });
    }
}

fn tag_label(ui: &mut egui::Ui, db: &mut DbBackend, tag_id: TagId) {
    let mut text = RichText::new(&db[tag_id].name);

    if let Some(category_id) = db.category_for_tag(tag_id) {
        text = text.color(Color32::from_rgb(
            db[category_id].color[0],
            db[category_id].color[1],
            db[category_id].color[2],
        ));
    }

    let response = ui.selectable_label(false, text);

    let description = &db[tag_id].description;
    if description.trim() != "" {
        response.on_hover_ui(|ui| {
            ui.label(description);
        });
    }
}
