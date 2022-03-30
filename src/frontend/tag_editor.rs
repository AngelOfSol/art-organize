use db::PieceId;
use egui::{PointerButton, ScrollArea, TextEdit, Ui};
use std::hash::Hash;

use crate::{backend::DbBackend, frontend::tag_label, ui_memory::MemoryExt};

pub fn tag_editor<IdSource>(ui: &mut Ui, id: IdSource, piece_id: PieceId, db: &mut DbBackend)
where
    IdSource: Hash + std::fmt::Debug,
{
    let memory_id = ui.make_persistent_id(id);

    ui.with_memory(memory_id, String::new, |ui, filter| {
        ui.add(TextEdit::singleline(filter).hint_text("Search"));

        let (added, unadded) = db
            .tags
            .keys()
            .partition::<Vec<_>, _>(|tag_id| db.piece_tags.contains(&(piece_id, *tag_id)));

        ui.horizontal(|ui| {
            ui.set_min_height(200.0);
            ScrollArea::vertical()
                .id_source(ui.make_persistent_id("unadded"))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(140.0);
                        let unadded = unadded
                            .into_iter()
                            .filter(|tag_id| {
                                filter.trim() == ""
                                    || db[tag_id].name.matches(filter.as_str()).count() > 0
                            })
                            .collect::<Vec<_>>();
                        for tag_id in unadded {
                            if tag_label(ui, db, tag_id).clicked_by(PointerButton::Secondary) {
                                db.piece_tags.insert((piece_id, tag_id));
                            }
                        }
                    });
                });

            ScrollArea::vertical()
                .id_source(ui.make_persistent_id("added"))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(140.0);
                        for tag_id in added {
                            if tag_label(ui, db, tag_id).clicked_by(PointerButton::Secondary) {
                                db.piece_tags.remove(&(piece_id, tag_id));
                            }
                        }
                    });
                });
        });
    });
}
