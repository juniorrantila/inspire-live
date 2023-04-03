use egui::{Response, Ui, Widget};

pub struct Display {
    pub content: String,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            content: "Lorem ipsum dolor sit amet".to_owned(),
        }
    }
}

impl Widget for &mut Display {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.label(&self.content)
    }
}
