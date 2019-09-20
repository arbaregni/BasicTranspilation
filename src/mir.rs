use sexpr::{Sexpr, SexprId, SexprKind};
use util::Error;
use manager::Manager;

pub struct MirExpr {
    fmt_str: String,
    handles: Vec<MirExpr>,
}
impl MirExpr {
    fn empty() -> MirExpr { MirExpr{ fmt_str: String::new(), handles: vec![] }}
}

pub enum MirStmnt {
    Malloc{width: usize},
    WriteAtFixed{at: usize, expr: MirExpr},
    InvokeFunc{call_site_id: usize, func_id: usize},
    IfBlock{predicate: MirExpr, if_stmnts: Vec<MirStmnt>, if_expr: MirExpr, else_stmnts: Vec<MirStmnt>, else_expr: MirExpr},
    WhileBlock{},
}

impl Manager {
    fn make_mir(&self, sexpr_id: SexprId, mir: &mut Vec<MirStmnt>) -> Result<MirExpr, Error> {
        use std::ops::Deref;
        match *self.sexpr(sexpr_id).kind.deref() {
            SexprKind::Declare {ref variable_pattern, expr, body} => {
                let at = self.prealloc();
                let expr = self.make_mir(*expr, mir);
                mir.push(MirStmnt::WriteAtFixed {at, expr});
                self.make_mir(*body, mir);
            },
            SexprKind::Assign {ref variable_pattern, expr} => {
                let expr = self.make_mir(*expr, mir);
                mir.push(MirStmnt::WriteAtFixed {at, expr});
            }
            SexprKind::IfSwitch {predicate, if_branch, else_branch} => {
                let predicate = self.make_mir(*predicate, mir);
                let if_stmnts = vec![];
                let if_expr = self.make_mir(*if_branch, &mut if_stmnts);
                let else_stmnts = vec![];
                let else_expr = self.make_mir(*else_branch, &mut else_stmnts);
                mir.push(MirStmnt::IfBlock{predicate, if_stmnts, if_expr, else_stmnts, else_expr});
            }
            SexprKind::WhileLoop {predicate, body} => {
                unimplemented!()
            }
            SexprKind::Block {ref statements} => {
                for ref statement in statements {
                    self.make_mir(**statement)?;
                }
            }
            SexprKind::StructGet { id:_, expr, field:_ } => {
                unimplemented!()
            }
            SexprKind::StructSet { id:_, expr, field:_, value } => {
                unimplemented!()
            }
            SexprKind::FuncDef { func_id } => {
                unimplemented!()
            }
            SexprKind::StructDef { id } => {
                unimplemented!()
            }
            SexprKind::Format { ref exprs } => {
                unimplemented!()
            }
            SexprKind::Other { ref opt_exprs } => {
                unimplemented!()
            }
            // we don't know these exist yet
            SexprKind::FuncCall {..} => {
                unimplemented!()
            }
            SexprKind::StructInit {..} => {
                unimplemented!()
            }
            SexprKind::Builtin {..} => {
                unimplemented!()
            }
            SexprKind::Identifier => {
                return MirExpr::ReadAtFixed{};
            }
            SexprKind::RealLiteral | SexprKind::IntegerLiteral => {
                unimplemented!()
            }
            SexprKind::StringLiteral => {
                unimplemented!()
            }
            SexprKind::BooleLiteral => {
                unimplemented!()
            },
        }
        Ok(MirExpr::empty())
    }
}

pub fn make_all_mir(m: &mut Manager) -> Result<Vec<MirSymbol>, Error> {

}