use std::ops::Index;

use sil::AstNode;
use sil::Garbage;
use sil::Node;
use sil::AST;

struct Layers(Vec<Layer>);
impl From<&AST> for Layers {
    fn from(ast: &AST) -> Self {
        let mut layers = Vec::new();
        for block in ast.view() {
            match block {
                &AstNode::Node(id) => {
                    layers.push(Layer::from(&ast[id]));
                }
                &AstNode::Garbage(id) => {
                    layers.push(Layer::from(&ast[id]));
                }
                AstNode::Reserve => unreachable!(),
            }
        }
        return Layers(layers);
    }
}

impl Index<usize> for Layers {
    type Output = Layer;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Align {
    pub fn from(s: &str) -> Option<Self> {
        match s {
            "left" => Some(Align::Left),
            "center" => Some(Align::Center),
            "right" => Some(Align::Right),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TextLayer {
    pub text: &'static str,
    pub font_size: f64,
    pub font_weight: FontWeight,
}

impl Default for TextLayer {
    fn default() -> Self {
        Self {
            text: "",
            font_size: 16.0,
            font_weight: FontWeight::Medium,
        }
    }
}

impl From<&Node> for TextLayer {
    fn from(value: &Node) -> Self {
        let mut res = Self::default();
        res.text = value.text();
        return res;
    }
}

#[derive(Debug, PartialEq)]
pub struct TitleLayer {
    pub text: &'static str,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub color: u32,
}

impl Default for TitleLayer {
    fn default() -> Self {
        Self {
            text: "",
            font_size: 24.0,
            font_weight: FontWeight::Bold,
            color: 0x00000000,
        }
    }
}

impl From<&Node> for TitleLayer {
    fn from(value: &Node) -> Self {
        let mut res = Self::default();
        res.text = value.text();
        return res;
    }
}

#[derive(Debug, PartialEq)]
pub enum Layer {
    Text(TextLayer),
    Title(TitleLayer),
    GarbageNode(Node),
    Garbage(Garbage),
}

impl From<&Node> for Layer {
    fn from(node: &Node) -> Self {
        match node.kind {
            "text" => Layer::Text(TextLayer::from(node)),
            "title" => Layer::Title(TitleLayer::from(node)),
            _ => Layer::GarbageNode(*node),
        }
    }
}

impl From<&Garbage> for Layer {
    fn from(garbage: &Garbage) -> Self {
        Self::Garbage(*garbage)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Light,
    Medium,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Monospace,
    Normal,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use sil::*;

    #[test]
    fn can_convert_simple_node() {
        let mut ast = AST::new();

        let node_slot = ast.reserve_slot();
        ast[node_slot] = ast.push_node(Node {
            kind: "title",
            attributes: &[],
            body: &[Token::Text("Foobar")],
        });

        let layers = Layers::from(&ast);
        assert_eq!(
            layers[0],
            Layer::Title(TitleLayer {
                text: "Foobar",
                font_size: 24.0,
                font_weight: FontWeight::Bold,
                color: 0x00000000,
            })
        );
    }
}
