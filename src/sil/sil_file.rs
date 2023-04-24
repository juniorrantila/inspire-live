mod lex;
mod parse;
mod typecheck;

pub use lex::*;
pub use parse::*;
pub use typecheck::*;

pub struct SilFile {
    pub content: &'static str,
}

impl SilFile {
    pub fn from(content: &'static str) -> Option<Self> {
        let tokens = lex(content);
        let ast = parse(&tokens);
        let _typechecked_ast = typecheck(&ast);

        return Some(SilFile {
            content
        });
    }
}
