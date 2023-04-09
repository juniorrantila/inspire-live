use std::ops::Index;
use std::ops::IndexMut;

use crate::lex::Token;

pub struct AST<'content> {
    nodes: Vec<Node<'content>>,
    garbages: Vec<Garbage<'content>>,
    attributes: Vec<Attribute<'content>>,

    ast: Vec<AstNode>,
}

impl<'content> AST<'content> {
    pub fn new() -> AST<'content> {
        AST {
            nodes: Vec::new(),
            garbages: Vec::new(),
            attributes: Vec::new(),
            ast: Vec::new(),
        }
    }

    fn push_node(&mut self, node: Node<'content>) -> AstNode {
        let index = self.nodes.len();
        self.nodes.push(node);
        return AstNode::Node(index);
    }

    fn push_garbage(&mut self, garbage: Garbage<'content>) -> AstNode {
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

impl<'content> Index<AstSlot> for AST<'content> {
    type Output = AstNode;
    fn index(&self, index: AstSlot) -> &Self::Output {
        &self.ast[index.0]
    }
}

impl<'content> IndexMut<AstSlot> for AST<'content> {
    fn index_mut(&mut self, index: AstSlot) -> &mut Self::Output {
        &mut self.ast[index.0]
    }
}

struct AstSlot(usize);

pub enum AstNode {
    Node(usize),
    Garbage(usize),
    Reserve,
}

struct Garbage<'content> {
    offending_tokens: &'content [Token<'content>],
    message: &'static str,
}

struct Node<'content> {
    kind: &'content str,
    attributes: &'content [Attribute<'content>],
    body: &'content str,
}

struct Attribute<'content> {
    name: &'content str,
    value: &'content str,
}

struct ParseValue<T> {
    value: T,
    consumed_tokens: usize,
}

pub fn parse<'content>(tokens: &'content [Token<'content>]) -> AST<'content> {
    let mut ast = AST::new();

    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        match token {
            Token::Number(_) => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..1],
                    message: "unexpected number",
                });
                i += 1;
            }
            Token::Text(_) => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..1],
                    message: "unexpected text",
                });
                i += 1;
            }
            Token::Quoted(_) => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..1],
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
                    offending_tokens: &tokens[i..1],
                    message: "unexpected quoted",
                });
                i += 1;
            }
            Token::Colon => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..1],
                    message: "unexpected colon",
                });
                i += 1;
            }
            Token::EqualSign => {
                ast.push_garbage(Garbage {
                    offending_tokens: &tokens[i..1],
                    message: "unexpected equal sign",
                });
                i += 1;
            }
        }
    }

    ast.push_node(Node {
        kind: tokens[0].text(),
        attributes: &[],
        body: "bar",
    });

    ast.push_garbage(Garbage {
        offending_tokens: &tokens[0..0],
        message: "foobar",
    });

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

fn parse_block<'content>(
    ast: &'content mut AST<'content>,
    tokens: &'content [Token<'content>],
) -> ParseBlock {
    let mut consumed_tokens = 0;
    let key = &tokens[0..3];
    match key {
        [Token::OpenBracket, Token::Text(kind), Token::CloseBracket] => {
            consumed_tokens += key.len();
            match parse_attributes(ast, &tokens[consumed_tokens..]) {
                ParseAttributes::Attributes {
                    value: attributes,
                    consumed_tokens: consumed,
                } => {
                    consumed_tokens += consumed;
                    match parse_body(ast, &tokens[consumed_tokens..]) {
                        ParseBody::Body {
                            value: body,
                            consumed_tokens: consumed,
                        } => ParseBlock::Node {
                            value: ast.push_node(Node {
                                kind,
                                attributes,
                                body,
                            }),
                            consumed_tokens: consumed_tokens + consumed,
                        },
                        ParseBody::Garbage {
                            value,
                            consumed_tokens: consumed,
                        } => ParseBlock::Garbage {
                            value: ast.push_garbage(value),
                            consumed_tokens: consumed_tokens + consumed,
                        },
                    }
                }
                ParseAttributes::Garbage {
                    value,
                    consumed_tokens: consumed,
                } => ParseBlock::Garbage {
                    value: ast.push_garbage(value),
                    consumed_tokens: consumed_tokens + consumed,
                },
            }
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

enum ParseAttributes<'ast, 'content> {
    Attributes {
        value: &'ast [Attribute<'content>],
        consumed_tokens: usize,
    },
    Garbage {
        value: Garbage<'content>,
        consumed_tokens: usize,
    },
}

fn parse_attributes<'ast, 'content>(
    ast: &'ast mut AST<'content>,
    tokens: &'content [Token<'content>],
) -> ParseAttributes<'ast, 'content> {
    let mut consumed_tokens = 0;

    let start_index = ast.attributes.len();

    // Parse attribute in loop.

    let end_index = ast.attributes.len();

    return ParseAttributes::Attributes {
        value: &ast.attributes[start_index..end_index],
        consumed_tokens,
    };
}

enum ParseAttribute<'content> {
    Attribute {
        value: Attribute<'content>,
        consumed_tokens: usize,
    },
    Garbage {
        value: Garbage<'content>,
        consumed_tokens: usize,
    },
}

fn parse_attribute<'content>(
    ast: &mut AST<'content>,
    tokens: &'content [Token<'content>],
) -> ParseAttribute<'content> {
    ParseAttribute::Garbage {
        value: Garbage {
            offending_tokens: &[],
            message: "",
        },
        consumed_tokens: 1,
    }
}

enum ParseBody<'content> {
    Body {
        value: &'content str,
        consumed_tokens: usize,
    },
    Garbage {
        value: Garbage<'content>,
        consumed_tokens: usize,
    },
}

fn parse_body<'content>(
    _ast: &mut AST<'content>,
    _tokens: &'content [Token<'content>],
) -> ParseBody<'content> {
    ParseBody::Garbage {
        value: Garbage {
            offending_tokens: &[],
            message: "",
        },
        consumed_tokens: 1,
    }
}
