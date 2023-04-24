use egui::RichText;
use egui::{Response, Ui, Widget };
use egui::{Align, Layout, Color32};
use sil::{AST, Token};
use slide::{Layer, Layers};

struct Slide {
    tokens: Vec<Token>,
    ast: AST,
    layers: Layers,
}

pub struct Display {
    pub content: String,
    slide: Option<Slide>
}

impl Default for Display {
    fn default() -> Self {
        Self {
            content: "Lorem ipsum dolor sit amet".to_owned(),
            slide: None,
        }
    }
}

impl Display {
    pub fn update(&mut self) {
        let mut slide = Slide {
            tokens: Vec::new(),
            ast: AST::new(),
            layers: Layers::default()
        };
        slide.tokens = unsafe {
            sil::lex(&*(self.content.as_str() as *const str))
        };
        slide.ast = sil::parse(&slide.tokens);
        slide.layers = Layers::from(&slide.ast);
        self.slide = Some(slide);
    }
}

impl Widget for &mut Display {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(slide) = &self.slide {
            if slide.layers.view().is_empty() {
                return ui.monospace(self.content.as_str());
            }

            let mut ui = ui.child_ui(ui.max_rect(), Layout::top_down(Align::Center));
            for layer in slide.layers.view() {
                match layer {
                    Layer::Title(layer) => {
                        ui.heading(RichText::new(layer.text).size(layer.font_size as f32));
                    }
                    Layer::Garbage(garbage) => {
                        ui.label(RichText::new(garbage.text()).color(Color32::RED));
                    }
                    Layer::GarbageNode(node) => {
                        ui.label(RichText::new(node.text()).color(Color32::RED));
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
