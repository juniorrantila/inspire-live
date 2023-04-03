use bevy::prelude::Resource;
use egui::{Align, Layout, TextStyle};

use crate::display::Display;
use crate::slides::Slides;

#[derive(Default, Resource)]
pub struct App {
    preview: Display,
    output: Display,

    slides: Slides,
}

impl App {
    pub fn output(&self) -> &Display {
        return &self.preview; // FIXME: Change to &self.output when we're able to commit messages.
    }

    pub fn update(&mut self, ctx: &egui::Context) {
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
                        egui::TextEdit::multiline(&mut self.preview.content)
                            .clip_text(true)
                            .font(TextStyle::Monospace)
                            .lock_focus(false),
                    )
                    .labelled_by(label.id);
                });
            });
        });
    }
}
