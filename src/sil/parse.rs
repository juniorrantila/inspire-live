use std::convert::From;
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

    pub fn view(&self) -> &[AstNode] {
        &self.ast
    }

    pub fn push_node(&mut self, node: Node) -> AstNode {
        let index = self.nodes.len();
        self.nodes.push(node);
        return AstNode::from(NodeId(index));
    }

    fn push_garbage(&mut self, garbage: Garbage) -> AstNode {
        let index = self.garbages.len();
        self.garbages.push(garbage);
        return AstNode::from(GarbageId(index));
    }

    pub fn reserve_slot(&mut self) -> AstSlot {
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

impl Index<NodeId> for AST {
    type Output = Node;
    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl Index<GarbageId> for AST {
    type Output = Garbage;
    fn index(&self, index: GarbageId) -> &Self::Output {
        &self.garbages[index.0]
    }
}

#[derive(Debug, PartialEq)]
pub struct AstSlot(usize);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeId(usize);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GarbageId(usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AstNode {
    Node(NodeId),
    Garbage(GarbageId),
    Reserve,
}

impl From<NodeId> for AstNode {
    fn from(id: NodeId) -> Self {
        Self::Node(id)
    }
}
impl From<GarbageId> for AstNode {
    fn from(id: GarbageId) -> Self {
        Self::Garbage(id)
    }
}

fn consolidate_tokens_into_string(tokens: &[Token]) -> &'static str {
    if tokens.is_empty() {
        return "";
    }
    if tokens.len() == 1 {
        return tokens[0].text();
    }

    let mut res = tokens[0].text();
    for token in &tokens[1..] {
        res = unsafe { res.get_unchecked(0..res.len() + token.text().len()) };
    }
    return res;
}

#[derive(Clone, Copy)]
pub struct Garbage {
    offending_tokens: *const [Token],
    message: &'static str,
}
impl Garbage {
    fn offending_tokens(&self) -> &[Token] {
        unsafe { &*self.offending_tokens } // Tokens are expected to outlive Garbage.
    }
    pub fn text(&self) -> &str {
        return consolidate_tokens_into_string(self.offending_tokens());
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

#[derive(Copy, Clone)]
pub struct Node {
    pub kind: &'static str,
    pub attributes: *const [Attribute],
    pub body: *const [Token],
}
impl Node {
    pub fn attributes(&self) -> &[Attribute] {
        unsafe { &*self.attributes } // Tokens are expected to outlive Node.
    }

    fn body(&self) -> &[Token] {
        unsafe { &*self.body } // Tokens are expected to outlive Node.
    }

    pub fn text(&self) -> &'static str {
        return consolidate_tokens_into_string(self.body());
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

pub struct Attribute {
    pub name: &'static str,
    pub value: *const [Token],
}
impl Attribute {
    fn value(&self) -> &[Token] {
        unsafe { &*self.value } // Tokens are expected to outlive Attribute.
    }

    pub fn value_text(&self) -> &str {
        return consolidate_tokens_into_string(self.value());
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

pub fn parse(complete_tokens: &[Token]) -> AST {
    let mut ast = AST::new();

    let mut tokens = complete_tokens;
    while !tokens.is_empty() {
        match tokens {
            [Token::Number(_), ..] => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[..1],
                    message: "unexpected number",
                });
                tokens = &tokens[1..];
            }
            [Token::Text(_), ..] => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[..1],
                    message: "unexpected text",
                });
                tokens = &tokens[1..];
            }
            [Token::Quoted(_), ..] => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[..1],
                    message: "unexpected quoted",
                });
                tokens = &tokens[1..];
            }
            [Token::OpenBracket(_), ..] => {
                let slot = ast.reserve_slot();
                let block = parse_block(&mut ast, tokens);
                ast[slot] = block.value;
                tokens = &tokens[block.consumed_tokens..];
            }
            [Token::CloseBracket(_), ..] => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[..1],
                    message: "unexpected quoted",
                });
                tokens = &tokens[1..];
            }
            [Token::Colon(_), ..] => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[..1],
                    message: "unexpected colon",
                });
                tokens = &tokens[1..];
            }
            [Token::EqualSign(_), ..] => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[..1],
                    message: "unexpected equal sign",
                });
                tokens = &tokens[1..];
            }
            [] => break,
        }
    }

    return ast;
}

struct ParseBlock {
    value: AstNode,
    consumed_tokens: usize,
}

fn parse_block(ast: &mut AST, tokens: &[Token]) -> ParseBlock {
    if tokens.len() < 3 {
        return ParseBlock {
            value: ast.push_garbage(Garbage {
                offending_tokens: tokens,
                message: "too few tokens",
            }),
            consumed_tokens: tokens.len(),
        };
    }

    let mut consumed_tokens = 0;
    match tokens {
        [Token::OpenBracket(_), Token::Text(kind), Token::CloseBracket(_), ..] => {
            consumed_tokens += 3;
            let attributes = parse_attributes(ast, &tokens[consumed_tokens..]);
            consumed_tokens += attributes.consumed_tokens;
            let attributes = attributes.value;

            let body = parse_body(ast, &tokens[consumed_tokens..]);
            consumed_tokens += body.consumed_tokens;
            let body = body.value;

            return ParseBlock {
                value: ast.push_node(Node {
                    kind,
                    attributes,
                    body,
                }),
                consumed_tokens,
            };
        }
        _ => ParseBlock {
            value: ast.push_garbage(Garbage {
                offending_tokens: tokens,
                message: "unexpected tokens",
            }),
            consumed_tokens: tokens.len(),
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
            [Token::Text(_), Token::EqualSign(_), ..] => {
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
        [Token::Text(name), Token::EqualSign(_), Token::Quoted(_), ..] => {
            ParseAttribute::Attribute {
                value: Attribute {
                    name,
                    value: &tokens[2..3],
                },
                consumed_tokens: 3,
            }
        }
        [Token::Text(name), Token::EqualSign(_), Token::Number(_), ..] => {
            ParseAttribute::Attribute {
                value: Attribute {
                    name,
                    value: &tokens[2..3],
                },
                consumed_tokens: 3,
            }
        }
        _ => ParseAttribute::Garbage {
            value: Garbage {
                offending_tokens: &tokens[0..1],
                message: "not an attribute",
            },
            consumed_tokens: 1,
        },
    }
}

struct ParseBody {
    value: *const [Token],
    consumed_tokens: usize,
}

fn parse_body(_ast: &mut AST, complete_tokens: &[Token]) -> ParseBody {
    let mut consumed_tokens = 0;

    let mut tokens = complete_tokens;
    while !tokens.is_empty() {
        match tokens {
            [Token::OpenBracket(_), Token::Text(_), Token::CloseBracket(_), ..] => break,
            _ => {
                consumed_tokens += 1;
                tokens = &complete_tokens[consumed_tokens..];
            }
        }
    }

    ParseBody {
        value: &complete_tokens[..consumed_tokens],
        consumed_tokens,
    }
}

unsafe impl Send for AST {}
unsafe impl Send for Node {}
unsafe impl Sync for Node {}
unsafe impl Sync for Garbage {}
unsafe impl Sync for Attribute {}

#[cfg(test)]
mod tests {
    use crate::parse::*;

    #[test]
    fn can_parse_block() {
        let tokens = [
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
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
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
            //
            Token::Text("foo"),
            Token::EqualSign("="),
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
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
            //
            Token::Text("foo"),
            Token::EqualSign("="),
            Token::Quoted("123"),
            //
            Token::Text("bar"),
            Token::EqualSign("="),
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

    #[test]
    fn can_parse_block_with_body() {
        let tokens = [
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
            //
            Token::Text("foo"),
            Token::EqualSign("="),
            Token::Quoted("123"),
            //
            Token::Text("bar"),
            Token::EqualSign("="),
            Token::Quoted("42"),
            //
            Token::Text("some text body"),
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
            body: &[Token::Text("some text body")],
        });
        let ast = parse(&tokens);

        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn can_parse_block_with_attributes_and_body() {
        let tokens = [
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
            //
            Token::Text("foo"),
        ];

        let mut expected_ast = AST::new();
        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "title",
            attributes: &[],
            body: &[Token::Text("foo")],
        });
        let ast = parse(&tokens);

        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn can_parse_multiple_simple_blocks() {
        let tokens = [
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
            //
            Token::OpenBracket("["),
            Token::Text("color"),
            Token::CloseBracket("]"),
        ];

        let mut expected_ast = AST::new();

        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "title",
            attributes: &[],
            body: &[],
        });

        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "color",
            attributes: &[],
            body: &[],
        });

        let ast = parse(&tokens);

        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn can_parse_multiple_blocks_with_bodies_and_attributes() {
        let tokens = [
            Token::OpenBracket("["),
            Token::Text("title"),
            Token::CloseBracket("]"),
            //
            Token::Text("weight"),
            Token::EqualSign("="),
            Token::Quoted("bold"),
            //
            Token::Text("Foobar"),
            //
            Token::OpenBracket("["),
            Token::Text("color"),
            Token::CloseBracket("]"),
            //
            Token::Text("name"),
            Token::EqualSign("="),
            Token::Quoted("black"),
            //
            Token::OpenBracket("["),
            Token::Text("lines"),
            Token::CloseBracket("]"),
            //
            Token::Text("some text content"),
            //
        ];

        let mut expected_ast = AST::new();

        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "title",
            attributes: &[Attribute {
                name: "weight",
                value: &[Token::Quoted("bold")],
            }],
            body: &[Token::Text("Foobar")],
        });

        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "color",
            attributes: &[Attribute {
                name: "name",
                value: &[Token::Quoted("black")],
            }],
            body: &[],
        });

        let node_slot = expected_ast.reserve_slot();
        expected_ast[node_slot] = expected_ast.push_node(Node {
            kind: "lines",
            attributes: &[],
            body: &[Token::Text("some text content")],
        });

        let ast = parse(&tokens);

        assert_eq!(ast, expected_ast);
    }
}
