use tokenizer::Token;
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone)]
pub struct SexprId { pub index: usize }
impl std::convert::From<usize> for SexprId {
    fn from(index: usize) -> SexprId {
        SexprId { index }
    }
}

#[derive(Debug)]
pub enum SexprKind {
    Declare{variable_pattern: String, expr: SexprId, body: SexprId},
    Assign{variable_pattern: String, expr: SexprId},
    IfSwitch{predicate: SexprId, if_branch: SexprId, else_branch: SexprId},
    WhileLoop{predicate: SexprId, body: SexprId},
    Block{statements: VecDeque<SexprId>},

    FuncDef{func_id: usize},
    FuncCall{func_id: usize, call_id: usize, exprs: VecDeque<SexprId>},

    StructDef {id: usize},
    StructInit{id: usize, exprs: VecDeque<SexprId>},
    StructGet{id: Option<usize>, expr: SexprId, field: String, },
    StructSet{id: Option<usize>, expr: SexprId, field: String, value: SexprId },

    Format{exprs: VecDeque<SexprId>},
    BuiltIn{id: usize, exprs: VecDeque<SexprId>},

    Other{opt_exprs: Option<VecDeque<SexprId>>},

    StringLiteral,
    IntegerLiteral,
    RealLiteral,
    BooleLiteral,
    Identifier,
}

#[derive(Debug)]
pub struct Sexpr {
    pub kind: Box<SexprKind>,
    pub token: Token,
}
impl Sexpr {
    pub fn new(kind: Box<SexprKind>, token: Token) -> Sexpr {
        Sexpr {
            kind,
            token,
        }
    }
}

