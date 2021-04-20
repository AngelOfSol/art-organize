use db::PieceId;
use imgui::{im_str, Selectable};
use itertools::Itertools;
use tag::ItemViewResponse;

use crate::app::widgets::{date::DATE_FORMAT, tag};

use super::{piece::PieceView, tag::TagView, GuiHandle, GuiView};

#[derive(Debug)]
pub struct Search {
    pub query_result: Vec<PieceId>,
    pub ascending: bool,
    pub sorted_by: SortedBy,
    pub show_single_tags: bool,
}

impl Default for Search {
    fn default() -> Self {
        Self {
            query_result: Vec::new(),
            ascending: true,
            sorted_by: SortedBy::Name,
            show_single_tags: false,
        }
    }
}

#[derive(Debug)]
pub enum SortedBy {
    Name,
    Price,
    Added,
}

impl GuiView for Search {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(
        &mut self,
        gui_handle: &GuiHandle,
        gui_state: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();

        if let Ok((_, parsed)) = search::parse_search(&gui_state.search.text) {
            self.query_result = parsed.execute(&db).collect();
        }

        ui.columns(6, im_str!("header"), true);

        if Selectable::new(im_str!("Name (# of Blobs)"))
            .selected(matches!(self.sorted_by, SortedBy::Name))
            .build(ui)
        {
            match self.sorted_by {
                SortedBy::Name => self.ascending = !self.ascending,
                SortedBy::Price | SortedBy::Added => {
                    self.sorted_by = SortedBy::Name;
                    self.ascending = true
                }
            }
        }
        ui.next_column();
        ui.text(im_str!("Media Type"));
        ui.next_column();
        ui.text(im_str!("Source Type"));
        ui.next_column();
        if Selectable::new(im_str!("Price"))
            .selected(matches!(self.sorted_by, SortedBy::Price))
            .build(ui)
        {
            match self.sorted_by {
                SortedBy::Price => self.ascending = !self.ascending,
                SortedBy::Name | SortedBy::Added => {
                    self.sorted_by = SortedBy::Price;
                    self.ascending = true
                }
            }
        }
        ui.next_column();
        ui.text(im_str!("Tip"));
        ui.next_column();
        if Selectable::new(im_str!("Date Added"))
            .selected(matches!(self.sorted_by, SortedBy::Added))
            .build(ui)
        {
            match self.sorted_by {
                SortedBy::Added => self.ascending = !self.ascending,
                SortedBy::Price | SortedBy::Name => {
                    self.sorted_by = SortedBy::Added;
                    self.ascending = true
                }
            }
        }

        ui.next_column();
        ui.separator();

        let query_result = self.query_result.iter().map(|id| (*id, &db[id]));
        let query_result = match self.sorted_by {
            SortedBy::Name => query_result.sorted_by_key(|(_, piece)| piece.name.to_lowercase()),
            SortedBy::Price => query_result.sorted_by_key(|(_, piece)| piece.base_price),
            SortedBy::Added => query_result.sorted_by_key(|(_, piece)| piece.added),
        };
        let query_result: Box<dyn Iterator<Item = _>> =
            if !self.ascending ^ matches!(self.sorted_by, SortedBy::Added) {
                Box::new(query_result.rev())
            } else {
                Box::new(query_result)
            };

        for (piece_id, piece) in query_result {
            let _id = ui.push_id(&im_str!("{}", piece_id));
            if Selectable::new(&im_str!("{}", piece.name))
                .span_all_columns(false)
                .build(ui)
            {
                gui_handle.goto(PieceView {
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

        if !self.query_result.is_empty() {
            let query_result = self.query_result.iter().map(|id| (id, &db[id]));

            let base_spent: i64 = query_result
                .clone()
                .map(|(_, piece)| piece.base_price.unwrap_or_default())
                .sum();
            let tip_spent: i64 = query_result
                .clone()
                .map(|(_, piece)| piece.tip_price.unwrap_or_default())
                .sum();
            let total_spent = base_spent + tip_spent;
            ui.text(&im_str!("Price Total: ${}", base_spent));
            ui.text(&im_str!("Tips: ${}", tip_spent));
            ui.text(&im_str!("Total Spent: ${}", total_spent));
            ui.text(&im_str!(
                "Tip Percentage: {}%",
                (tip_spent * 100).checked_div(total_spent).unwrap_or(0)
            ));

            let avg_price = query_result
                .clone()
                .filter_map(|(_, piece)| piece.base_price)
                .sum::<i64>()
                / query_result.clone().count() as i64;
            ui.text(&im_str!("Average Price: ${}", avg_price));

            let avg_tip = query_result
                .clone()
                .filter_map(|(_, piece)| piece.tip_price)
                .sum::<i64>()
                / query_result.clone().count() as i64;
            ui.text(&im_str!("Average Tip: ${}", avg_tip));

            ui.separator();
            ui.text(&im_str!("Piece Count: {}", query_result.clone().count()));
            ui.text(&im_str!(
                "Blob Count: {}",
                query_result
                    .clone()
                    .flat_map(|(id, _)| db.blobs_for_piece(*id))
                    .count()
            ));

            ui.separator();
            ui.checkbox(im_str!("Show 1-count tags"), &mut self.show_single_tags);

            imgui::ChildWindow::new(im_str!("Tags"))
                .draw_background(false)
                .build(ui, || {
                    for (count, tag_id) in query_result
                        .clone()
                        .flat_map(|(id, _)| db.tags_for_piece(*id))
                        .sorted()
                        .dedup_with_count()
                        .filter(|(count, _)| *count > 1 || self.show_single_tags)
                        .sorted_by_key(|(_, id)| &db[id].name)
                    {
                        match tag::item_view_with_count(ui, &db, tag_id, count) {
                            ItemViewResponse::None => {}
                            ItemViewResponse::Add => {}
                            ItemViewResponse::AddNegated => {}
                            ItemViewResponse::Open => {
                                gui_handle.goto(TagView {
                                    id: tag_id,
                                    edit: false,
                                });
                            }
                        }
                    }
                });
        }
    }
    fn label(&self) -> &'static str {
        "Search Screen"
    }
}
