use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use egui::{Color32, CtxRef, CursorIcon, Frame, Pos2, Stroke, Style, Vec2};
use egui::{menu};
use crate::i18n::Translator;

use dyngine_core::engine::{EngineInstance, ViewportRegion};

pub struct EngineApp {
    engine_instance: Rc<RefCell<EngineInstance>>,
    translator: Rc<Translator>,
    pub(crate) viewport_region: ViewportRegion,
    pub(crate) frame_time: Duration,
    pub(crate) fps_average_window: VecDeque<u32>,
}

impl EngineApp {
    pub fn new(engine_instance: Rc<RefCell<EngineInstance>>, translator: Rc<Translator>) -> Self {
        return EngineApp {
            engine_instance,
            translator,
            viewport_region: ViewportRegion::ZERO,
            frame_time: Duration::new(0, 0),
            fps_average_window: VecDeque::new(),
        };
    }
}

impl epi::App for EngineApp {
    #[profiling::function]
    fn update(&mut self, ctx: &CtxRef, _frame: &epi::Frame) {
        let engine_instance = self.engine_instance.borrow();

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
        egui::CentralPanel::default()
            .frame(Frame {
                margin: Vec2::new(0.0, 0.0),
                corner_radius: 0.0,
                fill: Color32::TRANSPARENT,
                stroke: Stroke {
                    color: Color32::TRANSPARENT,
                    width: 0.0,
                },
                ..Default::default()
            })
            .show(ctx, |ui| {
                let viewport_size_before_label = ui.available_size();

                // Hide cursor
                if engine_instance.should_grab_cursor() && engine_instance.window_state.has_focus() {
                    ctx.output().cursor_icon = CursorIcon::None;
                } else {
                    ctx.output().cursor_icon = CursorIcon::Default;
                }

                // render FPS label with average FPS over a time window of 60 frames
                let frame_time_nanos = self.frame_time.as_nanos();
                let label_pos;
                if frame_time_nanos != 0 {
                    let fps = (1_000_000_000.0 / (frame_time_nanos as f64)) as u32;
                    self.fps_average_window.push_back(fps);
                    if self.fps_average_window.len() > 60 {
                        self.fps_average_window.pop_front();
                    }
                    let fps_average = self.fps_average_window.iter().sum::<u32>() / self.fps_average_window.len() as u32;
                    label_pos = ui.label(format!("FPS: {}", fps_average)).rect.min;
                } else {
                    label_pos = Pos2::ZERO;
                }
                self.viewport_region = ViewportRegion {
                    x: label_pos.x,
                    y: label_pos.y,
                    width: viewport_size_before_label.x,
                    height: viewport_size_before_label.y,
                }
            });
    }

    fn name(&self) -> &str {
        "TestApp"
    }
}