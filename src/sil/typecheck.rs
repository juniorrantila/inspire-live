use crate::AstNode;
use crate::Node;
use crate::AST;

pub struct CheckedAst {
    nodes: Vec<CheckedNode>,
}
impl CheckedAst {
    fn eval<F>(&self, callback: F)
    where
        F: Fn(&CheckedNode) -> (),
    {
        for node in self.nodes.as_slice() {
            callback(node);
        }
    }
}

struct Types {}

pub fn typecheck(ast: &AST) -> CheckedAst {
    let mut view = ast.view();

    let types = Types {};

    while !view.is_empty() {
        match view {
            [AstNode::Node(id), ..] => {
                let _checked_node = typecheck_node(&types, &ast[*id]);
                view = &view[1..];
            }
            [AstNode::Garbage(id), ..] => {
                let _garbage = &ast[*id];
                view = &view[1..];
            }
            [AstNode::Reserve, ..] => unreachable!(),
            [] => break,
        }
    }

    CheckedAst {
        nodes: Vec::new()
    }
}

struct CheckedNode {}
fn typecheck_node(types: &Types, node: &Node) -> CheckedNode {
    match node {
        Node { kind: "title", .. } => typecheck_title(types, node),
        _ => todo!(),
    }
}

fn typecheck_title(_types: &Types, _node: &Node) -> CheckedNode {
    CheckedNode {}
}
