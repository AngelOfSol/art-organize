use egui::{CentralPanel, Layout, ScrollArea, Sense, Vec2};
use itertools::Itertools;

use crate::{backend::DbBackend, frontend::texture_storage::TextureStorage};

pub mod texture_storage;

pub enum Mode {
    Gallery,
}

pub struct Frontend {
    mode: Mode,
    thumbnails: TextureStorage,
}

impl Frontend {
    pub fn new(thumbnails: TextureStorage) -> Self {
        Self {
            mode: Mode::Gallery,
            thumbnails,
        }
    }
}

impl Frontend {
    pub fn update(&mut self, db: &mut DbBackend, ctx: &egui::CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
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
                                let response = ui
                                    .allocate_ui_with_layout(
                                        Vec2::new(266.0, 266.0),
                                        Layout::centered_and_justified(
                                            egui::Direction::RightToLeft,
                                        ),
                                        |ui| {
                                            ui.set_min_size([266.0; 2].into());
                                            if let Some(blob_id) = db
                                                .blobs_for_piece(piece_id)
                                                .sorted_by_key(|item| db[item].blob_type)
                                                .next()
                                            {
                                                if let Some(texture) =
                                                    self.thumbnails.get(blob_id, db)
                                                {
                                                    ui.image(
                                                        texture.id,
                                                        texture.scaled([256.0; 2]),
                                                    );
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
                                    println!("Clicked {:?}", piece_id);
                                }
                                response
                                    .on_hover_text(&format!("Description: {}", piece.description,));
                            }
                        });
                    }
                },
            );
        });
    }
}
