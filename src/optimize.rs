use manager::Manager;
use sexpr::{SexprId};

fn const_prop(m: &mut Manager, id: SexprId) {
    /*match id.kind(m) {
        SexprKind::Declare { ref variable_pattern, expr, body } => {
            //TODO variables:
            // self.use_var(m, "var", variable_pattern);
            self.make_dep(m, deps, "expr", expr, SameAs("var"));
            self.make_dep(m, deps, "body", body, Flexible);
        },
        SexprKind::Assign { variable_pattern:_, expr } => {
            //TODO variables:
            // self.use_var(m, "var", variable_pattern);
            self.make_dep(m, deps, "expr", expr, SameAs("var"));
        },
        SexprKind::IfSwitch { predicate, if_branch, else_branch } => {
            self.make_dep(m, deps, "predicate", predicate, MustBe(Number));
            self.make_util(m,      "temp", SameAs("if_branch"));
            self.make_dep(m, deps, "if_branch", if_branch, Flexible);
            self.make_dep(m, deps, "else_branch", else_branch, SameAs("if_branch"));
        },
        SexprKind::WhileLoop { predicate, body } => {
            self.make_dep(m, deps, "predicate", predicate, MustBe(Number));
            self.make_dep(m, deps, "body", body, Flexible);
        },
        SexprKind::Block { ref statements } => {
            //TODO how to handlle this? we want to depend on all of them
        },
        SexprKind::List { ref elements } => {
            //again multi dependence
            self.make_util(m, "list", MustBe(List));
        },
        SexprKind::ListGet { list, index } => {
            self.make_dep(m, deps, "list", list, MustBe(List));
            self.make_dep(m, deps, "index", index, MustBe(Number));
        },
        SexprKind::ListSet { list, index, elem } => {
            self.make_dep(m, deps, "list", list, MustBe(List));
            self.make_dep(m, deps, "index", index, MustBe(Number));
            self.make_dep(m, deps, "elem", elem, MustBe(Number));
        },
        SexprKind::FuncDef { id:_ } => {},
        SexprKind::FuncCall { func_id, call_id, ref exprs } => {
            self.make_util(m, "result", MustBe(Number));
        },
        SexprKind::StructDef { id } => {},
        SexprKind::StructInit { id, ref exprs } => {
            self.make_util(m, "begin_index", MustBe(Number));
        },
        SexprKind::StructGet { id, expr, ref field } => {
            self.make_dep(m, deps, "expr", expr, MustBe(Number));
        },
        SexprKind::StructSet { id, expr, ref field, value } => {
            self.make_dep(m, deps, "expr", expr, MustBe(Number));
            self.make_dep(m, deps, "value", value, MustBe(Number));
        },
        SexprKind::Format { ref exprs } => {},
        SexprKind::BuiltIn { id, ref exprs } => {},

        SexprKind::Other { .. } => panic!("should not be adding dependencies for an unresolved other"),
        //TODO variables:
        SexprKind::Identifier => {},

        // literals have no dependencies:
        SexprKind::StringLiteral | SexprKind::IntegerLiteral | SexprKind::IntegerLiteral | SexprKind::RealLiteral | SexprKind::BooleLiteral => {},
    } */
}