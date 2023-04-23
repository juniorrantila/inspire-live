use egui::RichText;
use egui::{Response, Ui, Widget };
use egui::{Align, Layout };
use sil::{AST, Token};
use slide::{Layer, Layers};

struct Slide {
    tokens: Vec<Token>,
    ast: AST,
    layers: Layers,
}

pub struct Display {
    pub content: String,
    old_content: String,
    slide: Option<Slide>
}

impl Default for Display {
    fn default() -> Self {
        Self {
            content: "Lorem ipsum dolor sit amet".to_owned(),
            old_content: "".to_owned(),
            slide: None,
        }
    }
}

impl Widget for &mut Display {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.content != self.old_content {
            self.old_content = self.content.clone();

            let mut slide = Slide {
                tokens: Vec::new(),
                ast: AST::new(),
                layers: Layers::default()
            };
            slide.tokens = unsafe {
                sil::lex(&*(self.content.as_str() as *const str)).unwrap_or(Vec::new())
            };
            slide.ast = sil::parse(&slide.tokens);
            slide.layers = Layers::from(&slide.ast);
            self.slide = Some(slide);
        }

        if let Some(slide) = &self.slide {
            if slide.layers.view().is_empty() {
                return ui.label(self.content.as_str());
            }

            let mut ui = ui.child_ui(ui.max_rect(), Layout::top_down(Align::Center));
            for layer in slide.layers.view() {
                match layer {
                    Layer::Title(layer) => {
                        ui.heading(RichText::new(layer.text).size(layer.font_size as f32));
                    }
                    Layer::Garbage(garbage) => {
                        ui.label(garbage.text());
                    }
                    Layer::GarbageNode(node) => {
                        ui.label(node.text());
                    }
                    Layer::Text(layer) => {
                        ui.label(RichText::new(layer.text).size(layer.font_size as f32));
                    }
                }
            }
        }

        ui.label("")
    }
}
