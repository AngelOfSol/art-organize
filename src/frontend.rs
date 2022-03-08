use db::{BlobId, PieceId, TagId};
use egui::{
    Button, CentralPanel, Color32, Frame, ImageButton, Label, Layout, RichText, ScrollArea, Sense,
    SidePanel, TopBottomPanel, Vec2,
};
use egui_demo_lib::easy_mark::easy_mark;
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::texture_storage::{ImageStatus, TextureStorage},
};

pub mod texture_storage;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Gallery,
    Piece((PieceId, Option<BlobId>)),
}

pub struct Frontend {
    history: Vec<Mode>,
    image_data: TextureStorage,
}

impl Frontend {
    pub fn new(thumbnails: TextureStorage) -> Self {
        Self {
            history: vec![Mode::Gallery],
            image_data: thumbnails,
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

        //
        match *self.history.last().unwrap() {
            Mode::Gallery => (),
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
                SidePanel::right("image_list")
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_min_width(276.0);
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered_justified(|ui| {
                                for blob_id in db
                                    .blobs_for_piece(piece_id)
                                    .sorted_by_key(|item| (db[item].blob_type, db[item].added))
                                {
                                    match self.image_data.thumbnail_for(blob_id, db) {
                                        ImageStatus::Available(texture) => {
                                            let response = ui.add(
                                                ImageButton::new(
                                                    texture.id,
                                                    texture.scaled([256.0; 2]),
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
                                                [256.0, 30.0],
                                                Button::new(&db[blob_id].file_name),
                                            );
                                        }
                                        ImageStatus::Loading => {
                                            ui.add_sized(
                                                [256.0; 2],
                                                Button::new(format!(
                                                    "Loading {}...",
                                                    db[blob_id].file_name
                                                )),
                                            );
                                        }
                                    }
                                }
                            });
                        });
                    });
            }
        }
        CentralPanel::default().show(ctx, |ui| match *self.history.last().unwrap() {
            Mode::Gallery => self.gallery_view(ui, db),
            Mode::Piece(state) => self.piece_view(ui, db, state),
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
                    ui.image(texture.id, texture.scaled(ui.available_size().into()));
                });
            }
        } else {
            ui.label("No Image");
        }
    }

    fn gallery_view(&mut self, ui: &mut egui::Ui, db: &mut DbBackend) {
        let row_size = (ui.available_width() / 266.0) as usize;
        ScrollArea::vertical().auto_shrink([false, true]).show_rows(
            ui,
            266.0,
            db.pieces().count() / row_size,
            |ui, row_range| {
                for row in db
                    .pieces()
                    .sorted_by_key(|(_, item)| item.added)
                    .rev()
                    .chunks(row_size)
                    .into_iter()
                    .skip(row_range.start)
                    .take(row_range.end - row_range.start)
                {
                    ui.horizontal(|ui| {
                        for (piece_id, piece) in row {
                            let blob_id = db
                                .blobs_for_piece(piece_id)
                                .sorted_by_key(|item| db[item].blob_type)
                                .next();
                            let response = ui
                                .allocate_ui_with_layout(
                                    Vec2::new(266.0, 266.0),
                                    Layout::centered_and_justified(egui::Direction::RightToLeft),
                                    |ui| {
                                        ui.set_min_size([266.0; 2].into());
                                        if let Some(blob_id) = blob_id {
                                            if let ImageStatus::Available(texture) =
                                                self.image_data.thumbnail_for(blob_id, db)
                                            {
                                                ui.image(texture.id, texture.scaled([256.0; 2]));
                                            }
                                        } else {
                                            ui.label("No Image");
                                        }
                                    },
                                )
                                .response;
                            let response = ui.interact(
                                response.rect,
                                ui.make_persistent_id(piece_id),
                                Sense::click(),
                            );
                            if response.clicked() {
                                self.history.push(Mode::Piece((piece_id, blob_id)));
                            }
                            response.on_hover_text(&format!("Description: {}", piece.description,));
                        }
                    });
                }
            },
        );
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
