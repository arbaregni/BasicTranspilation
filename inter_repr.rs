pub enum ReprKind {
    Const(),
    Var(),
    // replace with something for every possible expression we care about
    SimpleExpr{fmt: String, args: Vec<Rc<Repr>>}
}

pub struct Repr {
    variant: Rc<ReprKind>,
}

pub fn make_all_reprs() -> Vec<Repr> {

}