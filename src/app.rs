use self::actor::AppActor;
use crate::gui::GuiContext;
use actor::{AppTab, Inner, TabResult};
use imgui::{
    im_str, ImStr, MenuItem, MouseButton, Selectable, StyleColor, TabBar, TabBarFlags, TabItem,
    TreeNode, TreeNodeFlags, Ui, Window,
};
use piece_editor::PieceEditor;
use std::{ops::DerefMut, sync::Arc};
use winit::dpi::PhysicalSize;

pub mod actor;
pub mod piece_editor;

pub struct App {
    pub actor: Arc<AppActor>,
}

impl App {
    pub fn update(&mut self, _gui: &mut GuiContext) {
        let mut backend = self.actor.write();
        let Inner {
            // ipc,
            // handle,
            tabs,
            // image_cache,
            ..
        } = backend.deref_mut();
        for tab in tabs {
            tab.update();
        }
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
        let (mut backend, actor) = (self.actor.write(), &self.actor);
        let Inner {
            // ipc,
            // handle,
            backend,
            tabs,
            // image_cache,
            ..
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

        const EXPLORER_WIDTH: f32 = 400.0;

        let color = ui.push_style_color(StyleColor::WindowBg, [0.067, 0.067, 0.067, 1.0]);
        let mut close = None;

        let mut selected = None;

        Window::new(im_str!("Main"))
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .position([EXPLORER_WIDTH, 20.0], imgui::Condition::Always)
            .size(
                [window.width - EXPLORER_WIDTH, window.height - 20.0],
                imgui::Condition::Always,
            )
            .build(ui, || {
                TabBar::new(im_str!("Main Tabs"))
                    .reorderable(true)
                    .flags(TabBarFlags::AUTO_SELECT_NEW_TABS)
                    .build(ui, || {
                        for (idx, tab) in tabs.iter_mut().enumerate() {
                            let id = ui.push_id(idx as i32);

                            if let Some(label) = tab.label(&backend.db) {
                                let mut open = true;
                                TabItem::new(&im_str!("{}###tab", label))
                                    .opened(&mut open)
                                    .build(ui, || {
                                        if let TabResult::Kill = tab.render(&mut backend.db, ui) {
                                            close = Some(idx);
                                        } else {
                                            selected = Some(idx);
                                        }
                                    });

                                if !open {
                                    close = Some(idx);
                                }
                            }

                            id.pop(ui);
                        }
                    });
            });

        if let Some(idx) = close.take() {
            tabs.remove(idx);
            if selected == Some(idx) {
                selected = None;
            }
        }

        Window::new(im_str!("Explorer"))
            .movable(false)
            .resizable(false)
            .collapsible(true)
            .position([0.0, 20.0], imgui::Condition::Always)
            .size(
                [EXPLORER_WIDTH, window.height - 20.0],
                imgui::Condition::Always,
            )
            .build(ui, || {
                TreeNode::new(im_str!("open"))
                    .label(im_str!("Open Items"))
                    .default_open(true)
                    .build(ui, || {});
                for (idx, tab) in tabs.iter().enumerate() {
                    let label = if let Some(label) = tab.label(&backend.db) {
                        im_str!("{}", label)
                    } else {
                        close = Some(idx);
                        continue;
                    };
                    let selected = Some(idx) == selected;

                    match open_item(ui, selected, &label) {
                        OpenItemResult::None => {}
                        OpenItemResult::Close => {
                            close = Some(idx);
                        }
                        OpenItemResult::Clicked => {
                            // TODO make this work
                        }
                    };
                }
                TreeNode::new(im_str!("pieces"))
                    .label(im_str!("Pieces"))
                    .default_open(true)
                    .build(ui, || {
                        for (id, piece) in backend.query_pieces() {
                            let is_selected = selected
                                .and_then(|selected| tabs.get(selected))
                                .map(|item| {
                                    matches!(
                                        item,
                                        AppTab::Piece(PieceEditor { id: edit_id })
                                        if &id == edit_id
                                    )
                                })
                                .unwrap_or(false);

                            TreeNode::new(&im_str!("{}", piece.name))
                                .flags(
                                    TreeNodeFlags::SPAN_FULL_WIDTH | TreeNodeFlags::FRAME_PADDING,
                                )
                                .leaf(true)
                                .selected(is_selected)
                                .build(ui, || {});

                            if !is_selected
                                && ui.is_item_clicked(MouseButton::Left)
                                && ui.is_mouse_double_clicked(MouseButton::Left)
                            {
                                tabs.push(AppTab::Piece(PieceEditor { id }));
                            }
                        }
                    });
            });

        if let Some(idx) = close {
            tabs.remove(idx);
        }

        ui.show_default_style_editor();
        color.pop(ui);
    }
}

pub enum OpenItemResult {
    None,
    Close,
    Clicked,
}

fn open_item(ui: &Ui<'_>, selected: bool, label: &ImStr) -> OpenItemResult {
    let style = ui.clone_style();

    //ui.set_cursor_pos([ui, ui.cursor_pos()[1]]);
    ui.dummy([
        ui.content_region_avail()[0],
        ui.text_line_height() + style.frame_padding[1] * 2.0,
    ]);

    let clicked = ui.is_item_clicked(MouseButton::Left);

    let tl = ui.item_rect_min();
    let br = ui.item_rect_max();
    let draw_list = ui.get_window_draw_list();
    if ui.is_mouse_down(MouseButton::Left) && ui.is_item_hovered() {
        draw_list
            .add_rect(tl, br, style[StyleColor::HeaderActive])
            .filled(true)
            .thickness(1.0)
            .build();
    } else if selected {
        draw_list
            .add_rect(tl, br, style[StyleColor::Header])
            .filled(true)
            .thickness(1.0)
            .build();
    } else if ui.is_item_hovered() {
        draw_list
            .add_rect(tl, br, style[StyleColor::HeaderHovered])
            .filled(true)
            .thickness(1.0)
            .build();
    }

    let tl = [
        tl[0] + style.frame_padding[0],
        tl[1] + style.frame_padding[1],
    ];
    let br = [
        br[0] - style.frame_padding[0],
        br[1] - style.frame_padding[1],
    ];
    if ui.is_item_hovered() {
        let padding = -1.0;

        let button_height = br[1] - tl[1] - padding * 2.0;

        let tl = [br[0] - button_height, tl[1] + padding];
        let br = [br[0] - padding, br[1] - padding];

        let center = [(tl[0] + br[0]) / 2.0, (tl[1] + br[1]) / 2.0];

        let radius = (br[0] - tl[0]) / 2.0;
        if ui.is_mouse_hovering_rect(tl, br) {
            draw_list
                .add_circle(center, radius, style[StyleColor::ButtonActive])
                .num_segments(12)
                .filled(true)
                .thickness(1.0)
                .build();

            if ui.is_mouse_clicked(MouseButton::Left) {
                return OpenItemResult::Close;
            }
        }

        let radius = radius * 0.6;

        let center = [center[0] - 0.5, center[1] - 0.5];

        let tl = [center[0] - radius, center[1] - radius];
        let br = [center[0] + radius, center[1] + radius];

        let tr = [tl[0], br[1]];
        let bl = [br[0], tl[1]];

        draw_list
            .add_line(tl, br, style[StyleColor::Text])
            .thickness(1.0)
            .build();
        draw_list
            .add_line(tr, bl, style[StyleColor::Text])
            .thickness(1.0)
            .build();
    }

    let tl = [
        ui.cursor_pos()[0]
            + style.frame_padding[0] * 2.0
            + style.indent_spacing
            + ui.current_font_size(),
        br[1] - ui.text_line_height(),
    ];

    draw_list.add_text(tl, ui.style_color(StyleColor::Text), &label);

    if clicked {
        OpenItemResult::Clicked
    } else {
        OpenItemResult::None
    }
}
