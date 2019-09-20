use tokenizer::Token;
use sexpr::{SexprId, SexprKind};
use manager::Manager;
use util::Error;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Type {
    String,
    Int,
    Real,
    Boole,
    Void,
  //  Generic(&'static str), //TODO self-generic types
    List(Box<Type>),
    CustomType(String, usize), // the name of the type (for display purposes) and our id
}

impl Type {
    pub fn from_text(source: &str, token: Token, type_names: &Vec<String>) -> Result<Type, Error> {
        Type::from_text_primitive(token.get_text(source))
            .or(Manager::lookup_user_def_type(token.get_text(source), type_names))
            .ok_or(Error::new(format!("{} is not a recognized type", token.get_text(source)), token))
    }
    pub fn from_text_primitive(name: &str) -> Option<Type> {
        Some(match name {
            "string" => Type::String,
            "int" => Type::Int,
            "real" => Type::Real,
            "boole" => Type::Boole,
            "void" => Type::Void,
            typename if typename.len() >= 6 && &typename[0..5] == "list<" && &typename[typename.len() - 1..] == ">" => {
                Type::List(
                    Box::new(
                        Type::from_text_primitive(&typename[5..typename.len() - 1])?
                    )
                )
            }
            _ => return None,
        })
    }


    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn is_not(type0: &Type, type1: &Type) -> bool {
        match type0 {
            &Type::String => { if let &Type::String   = type1 { false } else { true }},
            &Type::Int    => { if let &Type::Int      = type1 { false } else { true }},
            &Type::Real   => { if let &Type::Real     = type1 { false } else { true }},
            &Type::Boole  => { if let &Type::Boole    = type1 { false } else { true }},
            &Type::Void   => { if let &Type::Void     = type1 { false } else { true }},
            &Type::CustomType(_, id0) => {
                if let &Type::CustomType(_, id1) = type1 {
                    id0 != id1
                } else {
                    true
                }
            }
            &Type::List(ref t0) => { if let &Type::List(ref t1) = type1 { Type::is_not(t0, t1) } else { true }}
        }
    }

    pub fn check_signature(signature0: &[Type], signature1: &[Type]) -> bool {
        if signature0.len() != signature1.len() {
            return false;
        }
        // compare each type in signature0 with the corresponding type in signature1
        for (type0, type1) in signature0.iter().zip(signature1.iter()) {
            if Type::is_not(type0, type1) {
                return false;
            }
        }
        true
    }
    pub fn check_futures_signature(signature0: &[FutureType], signature1: &[Type]) -> bool {
        if signature0.len() != signature1.len() {
            return false;
        }
        // compare each unwrapped type in signature0 with the corresponding type in signature1
        for (type0, type1) in signature0.iter().map(FutureType::unwrap).zip(signature1.iter()) {
            if Type::is_not(type0, type1) {
                return false;
            }
        }
        true
    }

    pub fn is_void(&self) -> bool {
        match *self {
            Type::Void => true,
            Type::List(ref t) => t.is_void(),
            _ => false,
        }
    }

}

impl ::std::fmt::Display for Type {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Type::String            => write!(f, "string"),
            Type::Int               => write!(f, "int"),
            Type::Real              => write!(f, "real"),
            Type::Boole             => write!(f, "boole"),
            Type::Void              => write!(f, "void"),
            Type::CustomType(ref n, _)  => write!(f, "{}", n),
            Type::List(ref cont)    => write!(f, "list<{}>", cont),
        }
    }
}

#[derive(Debug)]
pub enum FutureType {
    Proto(Token),
    Complete(Type),
}
impl FutureType {
    pub fn new(token: Token) -> FutureType {
        FutureType::Proto(token)
    }
    pub fn upgrade(&mut self, source: &str, type_names: &Vec<String>) -> Result<(), Error> {
        *self =
            if let FutureType::Proto(token) = *self {
                FutureType::Complete(Type::from_text(source, token, type_names)?)
            } else {
                return Ok(())
            };
        Ok(())
    }
    pub fn unwrap(&self) -> &Type {
        if let FutureType::Complete(ref typ) = self {
            typ
        } else {
            panic!("unwrapping Proto variant of FutureType: {:?}", self)
        }
    }
    pub fn unwrap_clone(&self) -> Type {
        self.unwrap().clone()
    }
}


/*

 */

impl Manager {
    /// convert the given sexpr from SexprKind::Other to something else
    fn realize_other(&mut self, sexpr_id: SexprId) -> Result<(), Error> {
        // during type checking, we also see if we can determine what unknowns are supposed to be
        let exprs = {
            if let SexprKind::Other { ref mut opt_exprs } = *self.sexpr_mut(sexpr_id).kind {
                opt_exprs.take().expect("other kind opt_exprs should be full when we're turning them into the real Sexpr")
            } else {
                return Ok(())
            }
        };
        let name = &self.text(sexpr_id).to_owned();

        let types = exprs
            .iter()
            .map(|expr: &SexprId| self.type_check(*expr))
            .collect::<Result<Vec<Type>, Error>>()?;

        let new_kind = {
            if let Some(func_id) = self.resolve_func(self.scope_of(sexpr_id), name, &types) {
                // are we a user defined function?
                let call_id = self.declare_call_site(func_id)?;
                SexprKind::FuncCall { func_id, call_id, exprs }
            } else if let Some(type_id) = self.resolve_struct_init(self.scope_of(sexpr_id), name, &types) {
                // are we a user defined type initializer?
                SexprKind::StructInit { id: type_id, exprs }
            } else if let Some(builtin_id) = self.builtin_manager.resolve_name(name, &types) {
                // are we a builtin?
                SexprKind::BuiltIn { id: builtin_id, exprs }
            } else {
                return Err(Error::new(format!(
                    "no operation found with name: `{}` and type signature: `{}`",
                    name,
                    types.iter().map(Type::to_string).collect::<Vec<String>>().join(" ")
                ), self.sexpr(sexpr_id).token))
            }
        };
        *self.sexpr_mut(sexpr_id).kind = new_kind;
        Ok(())
    }
    fn type_check(&mut self, sexpr_id: SexprId) -> Result<Type, Error> {
        // assume that our dependencies have been type checked
        self.realize_other(sexpr_id)?;
        let ret_type: Type =
            match & *self.sexpr(sexpr_id).kind {
                SexprKind::Declare {ref variable_pattern, expr, body} => {
                    let expr_type = self.type_check(*expr)?;
                    self.inform_var_type(self.scope_of(*body), variable_pattern, &expr_type);
                    self.type_check(*body)?
                },
                SexprKind::Assign {ref variable_pattern, expr} => {
                    let var_type = self
                        .resolve_variable(self.scope_of(sexpr_id), variable_pattern)
                        .and_then(|(scope_id, v_index)| self.scope(scope_id).variable_types[v_index].clone())
                        .expect("unresolved variable in typecheck");
                    let expr_type = self.type_check(*expr)?;
                    if Type::is_not(&var_type, &expr_type) {
                        return Err(Error::new(format!("assigning an expression of type {} to variable of type {}", expr_type, var_type), self.sexpr(sexpr_id).token));
                    }
                    var_type
                }
                SexprKind::IfSwitch {predicate, if_branch, else_branch} => {
                    let predicate_type = self.type_check(*predicate)?;
                    if Type::is_not(&predicate_type, &Type::Boole) {
                        return Err(Error::new(format!("if condition must be of type boole not {}", predicate_type), self.sexpr(sexpr_id).token));
                    }
                    let if_type= self.type_check(*if_branch)?;
                    let else_type = self.type_check(*else_branch)?;
                    if Type::is_not(&if_type, &else_type) {
                        return Err(Error::new_many(format!("branches must be of the same type. {} is not {}", if_type, else_type), vec![self.sexpr(*if_branch).token, self.sexpr(*else_branch).token]));
                    }
                    if_type
                }
                SexprKind::WhileLoop {predicate, body} => {
                    let predicate_type = self.type_check(*predicate)?;
                    if Type::is_not(&predicate_type, &Type::Boole) {
                        return Err(Error::new(format!("while condition must be of type boole, not {}", predicate_type), self.sexpr(*predicate).token));
                    }
                    self.type_check(*body)?
                }
                SexprKind::Block {ref statements} => {
                    let mut statement_type = Type::Void;
                    for i in 0..statements.len() {
                        statement_type = self.type_check(statements[i])?;
                    }
                    statement_type
                }
                SexprKind::FuncDef { func_id } => {
                    // make sure that the function's body matches up with the out type
                    let body_type = self.type_check(self.func_manager.body[*func_id])?;
                    if Type::is_not(&body_type, self.func_manager.out_type[*func_id].unwrap()) {
                        return Err(Error::new(format!("function body returns {} but function declaration states {}", body_type, self.func_manager.out_type[*func_id].unwrap()), self.sexpr(sexpr_id).token));
                    }
                    Type::Void
                },
                SexprKind::FuncCall { func_id, call_id: _, exprs: _ } => {
                    // exprs are type checked where we destruct the other kind
                    self.func_manager.out_type[*func_id].unwrap_clone()
                }
                SexprKind::StructDef { id: _ } => Type::Void,
                SexprKind::StructInit { id, exprs: _ } => {
                    // exprs are type checked when we evaluate what SexprKind other should be
                    Type::CustomType(self.udt_manager.name[*id].to_string(),*id)
                }
                SexprKind::StructGet { ref mut id, expr, ref field } => {
                    let expr_type = self.type_check(*expr)?;
                    if let Type::CustomType(_, struct_id) = &expr_type {
                        if !self.udt_manager.args[*struct_id].contains(&field) {
                            return Err(Error::new(format!("struct `{}` has no field with `{}`", expr_type, field), self.sexpr(sexpr_id).token));
                        }
                        let offset = self.udt_manager.get_field_offset(*struct_id, field);
                        let field_type = self.udt_manager.sgntr[*struct_id][offset].unwrap_clone();
                        *id = Some(*struct_id);
                        field_type
                    } else {
                        return Err(Error::new(format!("type `{}` is not a struct: cannot access field `{}`", expr_type, field), self.sexpr(sexpr_id).token));
                    }
                },
                SexprKind::StructSet { ref mut id, expr, ref field, value } => {
                    let expr_type = self.type_check(*expr)?;
                    if let Type::CustomType(_, struct_id) = &expr_type {
                        if !self.udt_manager.args[*struct_id].contains(&field) {
                            return Err(Error::new(format!("struct `{}` has no field named `{}`", expr_type, field), self.sexpr(sexpr_id).token));
                        }
                        let offset =self.udt_manager.get_field_offset(*struct_id, field);
                        let field_type = self.udt_manager.sgntr[*struct_id][offset].unwrap_clone();
                        *id = Some(*struct_id);
                        let value_type = self.type_check(*value)?;
                        if Type::is_not(&value_type, &field_type) {
                            return Err(Error::new(format!("field `{}` on struct `{}` is of type `{}`, not `{}`", field, expr_type, field_type, value_type), self.sexpr(sexpr_id).token))
                        }
                        Type::Void
                    } else {
                        return Err(Error::new(format!("cannot access field from type `{}`", expr_type), self.sexpr(*expr).token));
                    }
                },
                SexprKind::Format { ref mut exprs } => {
                    for ref mut expr in exprs {
                        self.type_check(**expr)?;
                    }
                    Type::String
                }
                SexprKind::Other { opt_exprs: _ } => panic!("we are type checking an Other (this should have been switched from an other before we got here)"),
                SexprKind::BuiltIn { id, exprs: _ } => {
                    // exprs are type checked when we resolve SexprKind::Other
                    self.builtin_manager.out_type[*id].clone()
                }
                SexprKind::Identifier       => {
                    self
                        .resolve_variable(self.scope_of(sexpr_id), self.text(sexpr_id))
                        .and_then(|(scope_id, v_index)| self.scope(self.scope_of(sexpr_id)).variable_types[v_index].clone())
                        .expect("typeless variable")
                }
                SexprKind::RealLiteral      => Type::Real,
                SexprKind::IntegerLiteral   => Type::Int,
                SexprKind::StringLiteral    => Type::String,
                SexprKind::BooleLiteral     => Type::Boole,
        };
        assert_eq!(self.sexpr_result_types.len(), sexpr_id.index);
        self.sexpr_result_types.push(ret_type.clone());
        Ok(ret_type)
    }
}

pub fn type_check_all(m: &mut Manager) -> Result<(), Error> {
     for ref sexpr_id in &m.top_level_sexprs { // iterate over all sexprs
        m.type_check(**sexpr_id)?;
    }
    Ok(())
}