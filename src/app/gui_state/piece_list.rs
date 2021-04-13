use imgui::{im_str, Selectable};

use crate::app::widgets::date::DATE_FORMAT;

use super::{GuiHandle, GuiView};

#[derive(Debug)]
pub struct PieceList;

impl GuiView for PieceList {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(&mut self, gui_handle: &GuiHandle, _: &super::InnerGuiState, ui: &imgui::Ui<'_>) {
        let db = gui_handle.db.read().unwrap();

        ui.columns(6, im_str!("header"), true);

        ui.text(im_str!("Name (# of Blobs)"));
        ui.next_column();
        ui.text(im_str!("Media Type"));
        ui.next_column();
        ui.text(im_str!("Source Type"));
        ui.next_column();
        ui.text(im_str!("Price"));
        ui.next_column();
        ui.text(im_str!("Tip"));
        ui.next_column();
        ui.text(im_str!("Date Added"));

        ui.columns(1, im_str!("unheader"), false);
        ui.separator();
        ui.columns(6, im_str!("piece list"), true);
        for (piece_id, piece) in db.pieces() {
            let _id = ui.push_id(&im_str!("{}", piece_id));
            if Selectable::new(&im_str!("{}", piece.name))
                .span_all_columns(false)
                .build(ui)
            {
                gui_handle.goto(super::piece::PieceView {
                    id: piece_id,
                    edit: false,
                })
            }
            ui.same_line();
            ui.text_colored(
                [0.4, 0.4, 0.4, 1.0],
                im_str!("{}", db.blobs_for_piece(piece_id).count()),
            );
            ui.next_column();

            ui.text(im_str!("{}", piece.media_type));
            ui.next_column();
            ui.text(im_str!("{}", piece.source_type));
            ui.next_column();
            if let Some(base_price) = piece.base_price {
                ui.text(im_str!("${}", base_price));
            }
            ui.next_column();
            if let Some(tip_price) = piece.tip_price {
                ui.text(im_str!("${}", tip_price));
            }
            ui.next_column();
            ui.text(im_str!("{}", piece.added.format(DATE_FORMAT)));
            ui.next_column();
        }

        ui.columns(1, im_str!("unpiece list"), false);
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.read().unwrap();
        let base_spent: i64 = db
            .pieces()
            .map(|(_, piece)| piece.base_price.unwrap_or_default())
            .sum();
        let tip_spent: i64 = db
            .pieces()
            .map(|(_, piece)| piece.tip_price.unwrap_or_default())
            .sum();
        let total_spent = base_spent + tip_spent;
        ui.text(&im_str!("Price Total: ${}", base_spent));
        ui.text(&im_str!("Tips: ${}", tip_spent));
        ui.text(&im_str!("Total Spent: ${}", total_spent));
        ui.text(&im_str!(
            "Tip Percentage: {}%",
            tip_spent * 100 / total_spent
        ));

        let avg_price = db
            .pieces()
            .filter_map(|(_, piece)| piece.base_price)
            .sum::<i64>()
            / db.pieces().count() as i64;
        ui.text(&im_str!("Average Price: ${}", avg_price));

        let avg_tip = db
            .pieces()
            .filter_map(|(_, piece)| piece.tip_price)
            .sum::<i64>()
            / db.pieces().count() as i64;
        ui.text(&im_str!("Average Tip: ${}", avg_tip));

        ui.separator();
        ui.text(&im_str!("Piece Count: {}", db.pieces().count()));
        ui.text(&im_str!("Blob Count: {}", db.blobs().count()));
    }
}
