use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use egui::{CtxRef, Frame, Style, Vec2};
use egui::{menu};
use crate::i18n::Translator;

use dyngine_core::engine::ViewportRegion;

pub struct TestApp {
    translator: Rc<Translator>,
    pub viewport_region: ViewportRegion,
    pub frame_time: Duration,
    pub fps_average_window: VecDeque<u32>
}

impl TestApp {
    pub fn new(translator: Rc<Translator>) -> Self {
        return TestApp {
            translator,
            viewport_region: ViewportRegion::ZERO,
            frame_time: Duration::new(0, 0),
            fps_average_window: VecDeque::new()
        };
    }
}

impl epi::App for TestApp {
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
        // ctx.style() has transparent background
        // This is to avoid erasing transparency where it is needed. (eg. viewport)
        // Only when we are certain that the element we are rendering should in fact remove
        // transparency, ether because it is not part of the viewport, or because it is an element
        // that should be able to occlude the viewport.
        // (eg. a menu, which can extend into parts of the window where the viewport could be)
        let style = Style::default();
        egui::TopBottomPanel::top("top_panel")
            .frame(Frame {
                margin: Vec2::new(8.0, 2.0),
                corner_radius: 0.0,
                fill: style.visuals.window_fill(),
                stroke: style.visuals.window_stroke(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                menu::bar(ui, |ui| {
                    ui.menu_button(self.translator.format("menubar-file", None).unwrap(), |ui| {
                        ui.menu_button(self.translator.format("menubar-file-new", None).unwrap(), |ui| {
                            if ui.button(self.translator.format("menubar-file-new-project", None).unwrap()).clicked() {
                                println!("New Project");
                            }
                        });
                        ui.menu_button(self.translator.format("menubar-file-open", None).unwrap(), |ui| {
                            if ui.button(self.translator.format("menubar-file-open-project", None).unwrap()).clicked() {
                                println!("Open Project");
                            }
                        });
                        if ui.button(self.translator.format("menubar-file-saveall", None).unwrap()).clicked() {
                            println!("Save All");
                        }
                    });
                    ui.menu_button(self.translator.format("menubar-edit", None).unwrap(), |ui| {
                        if ui.button(self.translator.format("menubar-edit-undo", None).unwrap()).clicked() {
                            println!("Undo");
                        }
                        if ui.button(self.translator.format("menubar-edit-redo", None).unwrap()).clicked() {
                            println!("Redo");
                        }
                        if ui.button(self.translator.format("menubar-edit-cut", None).unwrap()).clicked() {
                            println!("Cut");
                        }
                        if ui.button(self.translator.format("menubar-edit-copy", None).unwrap()).clicked() {
                            println!("Copy");
                        }
                        if ui.button(self.translator.format("menubar-edit-paste", None).unwrap()).clicked() {
                            println!("Paste");
                        }
                        if ui.button(self.translator.format("menubar-edit-delete", None).unwrap()).clicked() {
                            println!("Delete");
                        }
                        ui.menu_button(self.translator.format("menubar-edit-find", None).unwrap(), |ui| {
                            if ui.button(self.translator.format("menubar-edit-find-find", None).unwrap()).clicked() {
                                println!("Find");
                            }
                            if ui.button(self.translator.format("menubar-edit-find-replace", None).unwrap()).clicked() {
                                println!("Replace");
                            }
                            ui.separator();
                            if ui.button(self.translator.format("menubar-edit-find-findinfiles", None).unwrap()).clicked() {
                                println!("Find in Files");
                            }
                            if ui.button(self.translator.format("menubar-edit-find-findinfiles", None).unwrap()).clicked() {
                                println!("Find in Files");
                            }
                        });
                    });
                });
            });
        egui::SidePanel::left("left_panel")
            .frame(Frame {
                margin: Vec2::new(8.0, 2.0),
                corner_radius: 0.0,
                fill: style.visuals.window_fill(),
                stroke: style.visuals.window_stroke(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                egui::CollapsingHeader::new("Label 1")
                    .show(ui, |ui| {
                        ui.label("Sub Label 1");
                    });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            // query available_size before the label is rendered, because we want it to overlay the viewport
            let viewport_size = ui.available_size();

            // render FPS label with average FPS over a time window of 60 frames
            let frame_time_nanos = self.frame_time.as_nanos();
            if frame_time_nanos != 0 {
                let fps = (1_000_000_000.0 / (frame_time_nanos as f64)) as u32;
                self.fps_average_window.push_back(fps);
                if self.fps_average_window.len() > 60 {
                    self.fps_average_window.pop_front();
                }
                let fps_average = self.fps_average_window.iter().sum::<u32>() / self.fps_average_window.len() as u32;
                ui.label(format!("FPS: {}", fps_average));
            }

            let viewport_widget = egui::Label::new("");
            let response = ui.add_sized(viewport_size, viewport_widget);
            let response_rect = response.rect;
            self.viewport_region = ViewportRegion {
                x: response_rect.min.x,
                y: response_rect.min.y,
                width: response_rect.width(),
                height: response_rect.height(),
            }
        });
    }

    fn name(&self) -> &str {
        "TestApp"
    }
}