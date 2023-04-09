mod lex;
mod parse;

pub use lex::*;
pub use parse::*;

pub struct SilFile<'content> {
    pub layers: Vec<Layer<'content>>,
    pub content: &'content str,
}

impl <'content>SilFile<'content> {
    pub fn from(content: &'content str) -> Option<Self> {
        let layers: Vec<Layer> = Vec::new();

        let tokens = lex(content)?;
        let _ast = parse(&tokens);

        return Some(SilFile {
            layers,
            content
        });
    }
}

#[derive(Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right
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

pub enum Layer<'content> {
    Color(Color),
    Text(Text<'content>),
}

pub struct Color {
    value: u32,
}

pub struct Text<'content> {
    align: Align,
    content: &'content str,
    font_size: u32,
    font_weight: FontWeight,
    font_style: FontStyle,
}

impl Text<'_> {
    fn align(&self) -> Align { self.align }
    fn content(&self) -> &str { &self.content }
    fn font_size(&self) -> u32 { self.font_size }
    fn font_weight(&self) -> FontWeight { self.font_weight }
    fn font_style(&self) -> FontStyle { self.font_style }
}

impl <'content>Text<'content> {
    pub fn text(content: &'content str) -> Self {
        Self {
            align: Align::Center,
            content,
            font_size: 16,
            font_weight: FontWeight::Medium,
            font_style: FontStyle::Normal,
        }
    }

    pub fn header(content: &'content str) -> Self {
        Self {
            align: Align::Center,
            content,
            font_size: 24,
            font_weight: FontWeight::Bold,
            font_style: FontStyle::Normal,
        }
    }
}

#[derive(Clone, Copy)]
pub enum FontWeight {
    Light,
    Medium,
    Bold,
}

#[derive(Clone, Copy)]
pub enum FontStyle {
    Monospace,
    Normal
}
