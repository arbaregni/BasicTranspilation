use sexpr::*;
use manager::Manager;
use util::Error;
use scope::*;

impl Manager {
    ///creates the scope for this sexpr and all its children, returning ownership on a success.
    fn create_scope(&mut self, sexpr_id: SexprId, parent: ScopeId) -> Result<(), Error> {
        assert_eq!(self.sexpr_scopes.len(), sexpr_id.index);
        self.sexpr_scopes.push(parent);
        use std::ops::Deref;
        match *self.sexpr_mut(sexpr_id).kind.deref() {
            SexprKind::Declare {ref variable_pattern, expr, body} => {
                // create a child scope with the variable_name : declare_type binding
                let child = self.create_child(self.scope_of(sexpr_id));
                self.bind_variable(child, variable_pattern.to_string(), None);
                self.create_scope(expr, parent)?;
                self.create_scope(body, child)?;
            },
            SexprKind::Assign {ref variable_pattern, expr} => {
                if self.resolve_variable(self.scope_of(sexpr_id), variable_pattern).is_none(){
                    return Err(Error::new(format!("assigning to undeclared variable `{}`", variable_pattern), self.sexpr(sexpr_id).token));
                }
                self.create_scope(expr, parent)?;
            }
            SexprKind::IfSwitch {predicate,if_branch, else_branch} => {
                self.create_scope(predicate, parent)?;
                self.create_scope(if_branch, parent)?;
                self.create_scope(else_branch, parent)?;
            }
            SexprKind::WhileLoop {predicate, body} => {
                self.create_scope(predicate, parent)?;
                self.create_scope(body, parent)?;
            }
            SexprKind::Block {ref statements} => {
                for ref statement in statements {
                    self.create_scope(**statement, parent)?;
                }
            }
            SexprKind::StructGet { id:_, expr, field:_ } => {
                self.create_scope(expr, parent)?;
            }
            SexprKind::StructSet { id:_, expr, field:_, value } => {
                self.create_scope(expr, parent)?;
                self.create_scope(value, parent)?;
            }
            SexprKind::FuncDef { func_id } => {
                self.bind_func(self.scope_of(sexpr_id), func_id);
                // create our own parallel scoping business
                let new = Scope::new_to_vec(&mut self.all_scopes, None, true);
                self.bind_func(new, func_id); // it is visible inside its own scope for recursion
                for i in 0..(self.func_manager.args[func_id].len()) {
                    // bind all arguments
                    self.bind_variable(new,
                                      self.func_manager.args[func_id][i].clone(),
                                      Some(&self.func_manager.in_types[func_id][i].unwrap()) // unwrap is for the FutureType nonsense
                    );
                }
                self.create_scope(self.func_manager.body[func_id], new);
            },
            SexprKind::StructDef { id } => {
                self.bind_struct_init(self.scope_of(sexpr_id), id);
            }
            SexprKind::Format { ref exprs } => {
                for ref mut expr in exprs {
                    self.create_scope(**expr, parent)?;
                }
            }
            SexprKind::Other { ref opt_exprs } => {
                for ref mut expr in opt_exprs.unwrap() {
                    self.create_scope(*expr, parent)?;
                }
            }
            // we don't know these exist yet
            SexprKind::FuncCall {..} | SexprKind::StructInit {..} | SexprKind::BuiltIn {..} => panic!("scoping a expression that we shouldnt know about yet"),
            SexprKind::Identifier => {
                if self.resolve_variable(self.scope_of(sexpr_id), self.text(sexpr_id)).is_none() {
                    return Err(Error::new(format!("variable is undeclared"), self.sexpr(sexpr_id).token));
                }
            }
            SexprKind::RealLiteral | SexprKind::IntegerLiteral | SexprKind::StringLiteral | SexprKind::BooleLiteral => {},
        }
        Ok(())
    }
}

pub fn create_all_scopes(m: &mut Manager) -> Result<(), Error> {
    let global = Scope::new_to_vec(&mut m.all_scopes, None, false);
    for sexpr_id in m.top_level_sexprs {
        m.create_scope(sexpr_id, global)?;
    }
    Ok(())
}