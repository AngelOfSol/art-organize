use db::PieceId;
use egui::{PointerButton, ScrollArea, TextEdit, Ui};
use itertools::Itertools;
use std::hash::Hash;

use crate::{backend::DbBackend, frontend::tag, ui_memory::MemoryExt};

pub fn tag_editor<IdSource>(ui: &mut Ui, id: IdSource, piece_id: PieceId, db: &mut DbBackend)
where
    IdSource: Hash + std::fmt::Debug,
{
    let memory_id = ui.make_persistent_id(id);

    ui.with_memory(memory_id, String::new, |ui, filter| {
        let response = ui.add(TextEdit::singleline(filter).hint_text("Search"));
        let submitted = response.lost_focus() && ui.input().key_pressed(egui::Key::Enter);

        let (mut added, unadded) = db
            .tags
            .keys()
            .partition::<Vec<_>, _>(|tag_id| db.piece_tags.contains(&(piece_id, *tag_id)));
        let mut unadded = unadded
            .into_iter()
            .filter(|tag_id| {
                filter.trim() == "" || db[tag_id].name.matches(filter.as_str()).count() > 0
            })
            .sorted_by_key(|tag_id| &db[tag_id].name)
            .collect::<Vec<_>>();
        if submitted && unadded.len() == 1 {
            let tag_id = unadded.pop().unwrap();
            db.piece_tags.insert((piece_id, tag_id));
            added.push(tag_id);
            filter.clear();
            response.request_focus();
        }

        let (added, unadded) = (added, unadded);

        ui.horizontal(|ui| {
            ui.set_min_height(200.0);
            ScrollArea::vertical()
                .id_source(ui.make_persistent_id("unadded"))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(140.0);
                        for tag_id in unadded {
                            if tag::label(ui, db, tag_id).double_clicked_by(PointerButton::Primary)
                            {
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
                        for tag_id in added.into_iter().sorted_by_key(|tag_id| &db[tag_id].name) {
                            if tag::label(ui, db, tag_id).double_clicked_by(PointerButton::Primary)
                            {
                                db.piece_tags.remove(&(piece_id, tag_id));
                            }
                        }
                    });
                });
        });
    });
}
