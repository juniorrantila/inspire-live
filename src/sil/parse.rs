use std::fmt::Debug;
use std::ops::Index;
use std::ops::IndexMut;

use crate::lex::Token;

pub struct AST {
    nodes: Vec<Node>,
    garbages: Vec<Garbage>,

    attributes: Vec<Attribute>,

    ast: Vec<AstNode>,
}
impl AST {
    pub fn new() -> AST {
        AST {
            nodes: Vec::new(),
            garbages: Vec::new(),
            attributes: Vec::new(),
            ast: Vec::new(),
        }
    }

    fn push_node(&mut self, node: Node) -> AstNode {
        let index = self.nodes.len();
        self.nodes.push(node);
        return AstNode::Node(index);
    }

    fn push_garbage(&mut self, garbage: Garbage) -> AstNode {
        let index = self.garbages.len();
        self.garbages.push(garbage);
        return AstNode::Garbage(index);
    }

    fn reserve_slot(&mut self) -> AstSlot {
        let index = self.ast.len();
        self.ast.push(AstNode::Reserve);
        return AstSlot(index);
    }
}
impl PartialEq for AST {
    fn eq(&self, other: &Self) -> bool {
        // Ignore order of attributes since they're in nodes anyway.
        self.nodes == other.nodes && self.garbages == other.garbages && self.ast == other.ast
    }
}
impl Debug for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "AST {{ nodes: {:?}, garbages: {:?}, ast: {:?} }}",
            self.nodes, self.garbages, self.ast
        ))
    }
}

impl Index<AstSlot> for AST {
    type Output = AstNode;
    fn index(&self, index: AstSlot) -> &Self::Output {
        &self.ast[index.0]
    }
}

impl IndexMut<AstSlot> for AST {
    fn index_mut(&mut self, index: AstSlot) -> &mut Self::Output {
        &mut self.ast[index.0]
    }
}

#[derive(Debug, PartialEq)]
struct AstSlot(usize);

#[derive(Debug, PartialEq)]
pub enum AstNode {
    Node(usize),
    Garbage(usize),
    Reserve,
}

struct Garbage {
    offending_tokens: *const [Token],
    message: &'static str,
}
impl Garbage {
    fn offending_tokens(&self) -> &[Token] {
        unsafe { &*self.offending_tokens } // Tokens are expected to outlive Garbage.
    }
}
impl Debug for Garbage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "offending_tokens: {:?}, message: {:?}",
            self.offending_tokens(),
            self.message
        ))
    }
}
impl PartialEq for Garbage {
    fn eq(&self, other: &Self) -> bool {
        self.offending_tokens() == other.offending_tokens() && self.message == other.message
    }
}

struct Node {
    kind: &'static str,
    attributes: *const [Attribute],
    body: *const [Token],
}
impl Node {
    fn attributes(&self) -> &[Attribute] {
        unsafe { &*self.attributes } // Tokens are expected to outlive Node.
    }

    fn body(&self) -> &[Token] {
        unsafe { &*self.body } // Tokens are expected to outlive Node.
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.attributes() == other.attributes()
            && self.body() == other.body()
    }
}
impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Node {{ kind: {:?}, attributes: {:?}, body: {:?} }}",
            self.kind,
            self.attributes(),
            self.body()
        ))
    }
}

struct Attribute {
    name: &'static str,
    value: *const [Token],
}
impl Attribute {
    fn value(&self) -> &[Token] {
        unsafe { &*self.value } // Tokens are expected to outlive Attribute.
    }
}
impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value() == other.value()
    }
}
impl Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Attribute {{ name: {:?}, value: {:?} }}",
            self.name,
            self.value()
        ))
    }
}

struct ParseValue<T> {
    value: T,
    consumed_tokens: usize,
}

pub fn parse(tokens: &[Token]) -> AST {
    let mut ast = AST::new();

    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        match token {
            Token::Number(_) => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..i + 1],
                    message: "unexpected number",
                });
                i += 1;
            }
            Token::Text(_) => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..i + 1],
                    message: "unexpected text",
                });
                i += 1;
            }
            Token::Quoted(_) => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..i + 1],
                    message: "unexpected quoted",
                });
                i += 1;
            }
            Token::OpenBracket => {
                let slot = ast.reserve_slot();
                match parse_block(&mut ast, tokens) {
                    ParseBlock::Node {
                        value,
                        consumed_tokens,
                    } => {
                        ast[slot] = value;
                        i += consumed_tokens;
                    }
                    ParseBlock::Garbage {
                        value,
                        consumed_tokens,
                    } => {
                        ast[slot] = value;
                        i += consumed_tokens;
                    }
                }
            }
            Token::CloseBracket => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..i + 1],
                    message: "unexpected quoted",
                });
                i += 1;
            }
            Token::Colon => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..i + 1],
                    message: "unexpected colon",
                });
                i += 1;
            }
            Token::EqualSign => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..i + 1],
                    message: "unexpected equal sign",
                });
                i += 1;
            }
        }
    }

    return ast;
}

enum ParseBlock {
    Node {
        value: AstNode,
        consumed_tokens: usize,
    },
    Garbage {
        value: AstNode,
        consumed_tokens: usize,
    },
}

fn parse_block(ast: &mut AST, tokens: &[Token]) -> ParseBlock {
    let mut consumed_tokens = 0;
    let key = &tokens[0..3];
    match key {
        [Token::OpenBracket, Token::Text(kind), Token::CloseBracket] => {
            consumed_tokens += key.len();
            let attributes = parse_attributes(ast, &tokens[consumed_tokens..]);
            consumed_tokens += attributes.consumed_tokens;
            let attributes = attributes.value;

            let body = parse_body(ast, &tokens[consumed_tokens..]);
            consumed_tokens += body.consumed_tokens();
            let body = body.body();

            return ParseBlock::Node {
                value: ast.push_node(Node {
                    kind,
                    attributes,
                    body,
                }),
                consumed_tokens,
            };
        }
        _ => ParseBlock::Garbage {
            value: ast.push_garbage(Garbage {
                offending_tokens: key,
                message: "unexpected tokens",
            }),
            consumed_tokens: key.len(),
        },
    }
}

struct ParseAttributes {
    value: *const [Attribute],
    consumed_tokens: usize,
}

fn parse_attributes(ast: &mut AST, complete_tokens: &[Token]) -> ParseAttributes {
    let mut consumed_tokens = 0;
    let attributes_start = ast.attributes.len();

    let mut tokens = complete_tokens;
    while !tokens.is_empty() {
        match tokens {
            [Token::Text(_), Token::EqualSign, ..] => {
                let attribute = parse_attribute(ast, tokens);
                consumed_tokens += attribute.consumed_tokens();
                tokens = &complete_tokens[consumed_tokens..];
                if let ParseAttribute::Attribute {
                    value,
                    consumed_tokens: _,
                } = attribute
                {
                    ast.attributes.push(value);
                }
            }
            _ => break,
        }
    }

    let attributes_end = ast.attributes.len();
    return ParseAttributes {
        value: &ast.attributes[attributes_start..attributes_end],
        consumed_tokens,
    };
}

enum ParseAttribute {
    Attribute {
        value: Attribute,
        consumed_tokens: usize,
    },
    Garbage {
        value: Garbage,
        consumed_tokens: usize,
    },
}
impl ParseAttribute {
    fn consumed_tokens(&self) -> usize {
        return match self {
            Self::Attribute {
                value: _,
                consumed_tokens,
            } => *consumed_tokens,
            Self::Garbage {
                value: _,
                consumed_tokens,
            } => *consumed_tokens,
        };
    }
}

fn parse_attribute(_ast: &mut AST, tokens: &[Token]) -> ParseAttribute {
    match tokens {
        [Token::Text(name), Token::EqualSign, Token::Quoted(_), ..]=> ParseAttribute::Attribute {
            value: Attribute {
                name,
                value: &tokens[2..3],
            },
            consumed_tokens: 3,
        },
        _ => ParseAttribute::Garbage {
            value: Garbage {
                offending_tokens: &tokens[0..1],
                message: "not an attribute",
            },
            consumed_tokens: 1,
        },
    }
}

enum ParseBody {
    Body {
        value: *const [Token],
        consumed_tokens: usize,
    },
    Garbage {
        value: Garbage,
        consumed_tokens: usize,
    },
}

impl ParseBody {
    fn consumed_tokens(&self) -> usize {
        return match self {
            ParseBody::Body {
                value: _,
                consumed_tokens,
            } => *consumed_tokens,
            ParseBody::Garbage {
                value: _,
                consumed_tokens,
            } => *consumed_tokens,
        };
    }

    fn body(self) -> *const [Token] {
        match self {
            Self::Body {
                value,
                consumed_tokens: _,
            } => value,
            _ => &[],
        }
    }
}

fn parse_body(_ast: &mut AST, _tokens: &[Token]) -> ParseBody {
    ParseBody::Garbage {
        value: Garbage {
            offending_tokens: &[],
            message: "",
        },
        consumed_tokens: 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::*;

    #[test]
    fn can_parse_block() {
        let tokens = [
            Token::OpenBracket,
            Token::Text("title"),
            Token::CloseBracket,
        ];

        let mut expected_ast = AST::new();
        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "title",
            attributes: &[],
            body: &[],
        });

        let ast = parse(&tokens);
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn can_parse_block_with_single_attribute() {
        let tokens = [
            Token::OpenBracket,
            Token::Text("title"),
            Token::CloseBracket,
            //
            Token::Text("foo"),
            Token::EqualSign,
            Token::Quoted("123"),
        ];

        let mut expected_ast = AST::new();
        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "title",
            attributes: &[Attribute {
                name: "foo",
                value: &[Token::Quoted("123")],
            }],
            body: &[],
        });
        let ast = parse(&tokens);

        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn can_parse_block_with_two_attributes() {
        let tokens = [
            Token::OpenBracket,
            Token::Text("title"),
            Token::CloseBracket,
            //
            Token::Text("foo"),
            Token::EqualSign,
            Token::Quoted("123"),
            //
            Token::Text("bar"),
            Token::EqualSign,
            Token::Quoted("42"),
        ];

        let mut expected_ast = AST::new();
        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "title",
            attributes: &[
                Attribute {
                    name: "foo",
                    value: &[Token::Quoted("123")],
                },
                Attribute {
                    name: "bar",
                    value: &[Token::Quoted("42")],
                },
            ],
            body: &[],
        });
        let ast = parse(&tokens);

        assert_eq!(ast, expected_ast);
    }
}
