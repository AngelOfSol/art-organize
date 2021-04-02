use crate::gui::GuiContext;
use crate::layout::{Column, Dimension, LayoutRectangle, Row};
use glam::Vec2;
use imgui::{im_str, Ui, Window};
use std::sync::mpsc;
use std::{collections::HashMap, path::PathBuf};
use winit::dpi::PhysicalSize;

pub struct FirstTime {
    rx: mpsc::Receiver<PathBuf>,
    tx: mpsc::Sender<PathBuf>,
}

impl Default for FirstTime {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { rx, tx }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Layout {
    Centered,
    Other(usize),
}

impl FirstTime {
    pub fn update(&mut self, _: &mut GuiContext) -> Option<PathBuf> {
        self.rx.try_recv().ok()
    }

    pub fn render(&mut self, ui: &Ui<'_>, window: PhysicalSize<f32>) {
        let layout = {
            let mut layout_data = HashMap::new();
            let layout = Column::default()
                .push(Layout::Other(0), Dimension::Flex(1.0))
                .push(
                    Row::default()
                        .push(Layout::Other(1), Dimension::Flex(1.0))
                        .push(Layout::Centered, Dimension::Flex(1.0))
                        .push(Layout::Other(2), Dimension::Flex(1.0)),
                    Dimension::Flex(1.0),
                )
                .push(Layout::Other(3), Dimension::Flex(1.0));

            layout.layout(
                LayoutRectangle {
                    position: Vec2::ZERO,
                    size: Vec2::new(window.width, window.height),
                },
                &mut layout_data,
            );

            layout_data
        };

        Window::new(im_str!("New Database"))
            .movable(false)
            .resizable(false)
            .collapsible(true)
            .position(
                layout[&Layout::Centered].position.into(),
                imgui::Condition::Always,
            )
            .size(
                layout[&Layout::Centered].size.into(),
                imgui::Condition::Always,
            )
            .build(ui, || {
                if ui.button(im_str!("New Database")) {
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_folder().await {
                            tx.send(file.path().to_path_buf()).unwrap();
                        }
                    });
                }
            });
    }
}
