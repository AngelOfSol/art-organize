use self::actor::AppActor;
use crate::{consts::WINDOW_HEIGHT, gui::GuiContext};
use actor::AppState;
use imgui::{im_str, Ui, Window};
use std::sync::Arc;

pub mod actor;

pub struct Changed<T> {
    pub dirty: bool,
    value: T,
}

impl<T> Changed<T> {
    fn clean(value: T) -> Self {
        Self {
            dirty: false,
            value,
        }
    }
}

impl<T> std::ops::Deref for Changed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> std::ops::DerefMut for Changed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.value
    }
}

pub struct App {
    pub actor: Arc<AppActor>,
}

impl App {
    pub fn update(&mut self, gui: &mut GuiContext) {
        // while let Some(Some(item)) = self.ipc.recv().now_or_never() {
        //     match item {
        //         SubCommand::Init { path } => {}
        //         SubCommand::Add { path, .. } => match &mut self.state {
        //             AppState::Adding { piece, blobs } => {}
        //             AppState::None => {
        //                 self.state = AppState::Adding {
        //                     piece: Piece {
        //                         name: path
        //                             .file_name()
        //                             .and_then(|s| s.to_str())
        //                             .unwrap()
        //                             .to_owned(),
        //                         ..Piece::default()
        //                     },
        //                     blobs: vec![],
        //                 }
        //             }
        //         },
        //         SubCommand::Contextual { subcmd } => {}
        //         SubCommand::Gui => {}
        //         SubCommand::ResetConfig => {}
        //     }
        // }
    }

    pub fn render(&mut self, ui: &Ui<'_>) {
        let backend = self.actor.test();

        ui.show_default_style_editor();

        Window::new(im_str!("Pieces"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .position([0.0, 0.0], imgui::Condition::Always)
            .size([240.0, WINDOW_HEIGHT], imgui::Condition::Always)
            .build(ui, || match &backend.state {
                AppState::Adding { piece, blobs } => {
                    if ui.small_button(im_str!("New Piece")) {
                        ui.open_popup(im_str!("Overwrite Confirm"));
                    }

                    ui.popup_modal(im_str!("Overwrite Confirm"))
                        .collapsible(false)
                        .movable(false)
                        .resizable(false)
                        .always_auto_resize(true)
                        .build(|| {
                            if ui.small_button(im_str!("Overwrite")) {
                                ui.close_current_popup();
                            }
                            ui.same_line(0.0);

                            if ui.small_button(im_str!("Cancel")) {
                                ui.close_current_popup();
                            }
                        });

                    ui.text(&im_str!("{}", piece.name));
                    ui.text(&im_str!("Blob Count: {}", blobs.len()));
                }
                AppState::None => {
                    if ui.small_button(im_str!("New Piece")) {
                        self.actor.request_new_piece();
                    }

                    for piece in backend.backend.query_pieces() {
                        ui.text(&im_str!("{}", piece.name));
                    }
                }
            });
    }
}
