use egui::{Align, Layout, Widget};

pub struct Slides {
    file_names: Vec<String>,
    selected_slide: usize,
}

impl Default for Slides {
    fn default() -> Self {
        Self {
            file_names: ["Foo".to_owned(), "Bar".to_owned(), "Baz".to_owned()].to_vec(),
            selected_slide: 0,
        }
    }
}

impl Widget for &mut Slides {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            for (i, file) in self.file_names.iter().enumerate() {
                if ui
                    .selectable_label(self.selected_slide == i, file.as_str())
                    .clicked()
                {
                    self.selected_slide = i;
                }
            }
        })
        .response
    }
}
