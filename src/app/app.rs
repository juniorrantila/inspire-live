use bevy::prelude::Resource;
use egui::{Align, Layout, TextStyle, Label};

use root_path::RootPath;
use display::Display;
use slides::Slides;

#[derive(Resource)]
pub struct App {
    root: RootPath,

    preview: Display,
    output: Display,

    slides: Slides,

    content: String,
}

unsafe impl Send for App {}

impl App {
    pub fn from(root: RootPath) -> Self {
        Self {
            root,
            preview: Display::default(),
            output: Display::default(),
            slides: Slides::default(),

            content: "".to_owned(),
        }
    }

    pub fn output(&self) -> &Display {
        return &self.preview; // FIXME: Change to &self.output when we're able to commit messages.
    }

    fn update_slides_if_needed(&mut self) {
        self.slides.update_if_needed();
        if self.content != self.preview.content {
            self.preview.content = self.content.clone();
            self.preview.update();
        }
    }

    pub fn draw_control_window(&mut self, ctx: &egui::Context) {
        self.update_slides_if_needed();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let available_size = ui.available_size();
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    let label = ui.heading("preview");
                    ui.add_sized(available_size / 2.0, &mut self.preview)
                        .labelled_by(label.id);
                });
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    let label = ui.heading("output");
                    ui.add_sized(available_size / 2.0, &mut self.output)
                        .labelled_by(label.id);
                });
            });

            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    let available_size = ui.available_size();
                    let width = available_size.x / 4.0;
                    let height = available_size.y;
                    let label = ui.heading("slides");
                    ui.add_sized([width, height], &mut self.slides)
                        .labelled_by(label.id);
                });

                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    let label = ui.heading("edit");
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.content)
                            .clip_text(true)
                            .font(TextStyle::Monospace)
                            .lock_focus(false),
                    )
                    .labelled_by(label.id);
                });
            });
        });
    }

    pub fn draw_display_window(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.add_sized(
                    ui.available_size(),
                    &mut self.output
                );
            });
        });
    }
}
