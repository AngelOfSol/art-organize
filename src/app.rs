use self::{actor::AppActor, piece_editor::PieceEditor};
use crate::gui::GuiContext;
use actor::{Inner, TabResult};
use db::Db;
use imgui::{
    im_str, ChildWindow, MenuItem, Selectable, StyleColor, StyleVar, TabBar, TabBarFlags, Ui,
    Window,
};
use std::{ops::DerefMut, sync::Arc};
use winit::dpi::PhysicalSize;

pub mod actor;
pub mod piece_editor;

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

    pub fn render(&mut self, ui: &Ui<'_>, window: PhysicalSize<f32>) {
        let mut flag = false;
        {
            let (mut backend, actor) = (self.actor.write(), &self.actor);
            let Inner {
                ipc,
                handle,
                backend,
                tabs,
                image_cache,
            } = backend.deref_mut();

            ui.main_menu_bar(|| {
                ui.menu(im_str!("File"), true, || {
                    if MenuItem::new(im_str!("New Piece")).build(ui) {
                        actor.request_new_piece();
                    }
                });
                ui.menu(im_str!("Edit"), true, || {
                    if MenuItem::new(im_str!("Undo"))
                        .enabled(backend.db.can_undo())
                        .build(ui)
                    {
                        backend.db.undo();
                    }
                    if MenuItem::new(im_str!("Redo"))
                        .enabled(backend.db.can_redo())
                        .build(ui)
                    {
                        backend.db.redo();
                    }
                });
            });

            Window::new(im_str!("Explorer"))
                .movable(false)
                .resizable(false)
                .collapsible(false)
                .position([0.0, 20.0], imgui::Condition::Always)
                .size([240.0, window.height - 20.0], imgui::Condition::Always)
                .build(ui, || {
                    for piece in backend.query_pieces() {
                        ui.bullet_text(&im_str!("{}", piece.name));
                    }
                });

            let color = ui.push_style_color(StyleColor::WindowBg, [0.067, 0.067, 0.067, 1.0]);
            let mut close = None;

            Window::new(im_str!("Main"))
                .movable(false)
                .resizable(false)
                .collapsible(false)
                .no_decoration()
                .position([240.0, 20.0], imgui::Condition::Always)
                .size(
                    [window.width - 240.0, window.height - 20.0],
                    imgui::Condition::Always,
                )
                .build(ui, || {
                    TabBar::new(im_str!("Main Tabs"))
                        .reorderable(true)
                        .flags(TabBarFlags::AUTO_SELECT_NEW_TABS)
                        .build(ui, || {
                            for (idx, tab) in tabs.iter_mut().enumerate() {
                                let id = ui.push_id(idx as i32);
                                if let TabResult::Kill = tab.render(&mut backend.db, ui) {
                                    close = Some(idx);
                                }

                                id.pop(ui);
                            }
                        });
                });
            if let Some(idx) = close {
                tabs.remove(idx);
            }

            color.pop(ui);

            let mut t = true;
            ui.show_demo_window(&mut t);
        }
    }
}
