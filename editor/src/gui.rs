use std::ops::Sub;
use std::rc::Rc;
use egui::{CtxRef, Frame, Rect, Style, Vec2};
use egui::{menu};
use crate::i18n::Translator;

pub struct TestApp {
    translator: Rc<Translator>,
    pub viewport_rect: Rect,
}

impl TestApp {
    pub fn new(translator: Rc<Translator>) -> Self {
        return TestApp { translator, viewport_rect: Rect::NOTHING };
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
            let viewport_size = ui.available_size();
            let viewport_widget = egui::Label::new("");
            let response = ui.add_sized(viewport_size, viewport_widget);
            let response_rect = response.rect;
            let viewport_rect = Rect::from_min_max(response_rect.min, response_rect.max.sub(response_rect.min).to_pos2());
            self.viewport_rect = viewport_rect;
        });
    }

    fn name(&self) -> &str {
        "TestApp"
    }
}