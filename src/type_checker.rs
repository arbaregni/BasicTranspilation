use tokenizer::Token;
use scoper::Scope;
use sexprizer::{Sexpr, SexprKind};
use functionizer::FunctionManager;
use lang_consts;
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
}

impl Type {
    pub fn from_text(source: &str, token: Token) -> Result<Type, Error> {
        Type::from_text_helper(token.get_text(source)).ok_or(Error::new(format!("{} is not a recognized type", token.get_text(source)), token))
    }
    fn from_text_helper(name: &str) -> Option<Type> {
        Some(match name {
            "string" => Type::String,
            "int"    => Type::Int,
            "real"   => Type::Real,
            "boole"  => Type::Boole,
            "void"   => Type::Void,
            typename if typename.len() >= 6 && &typename[0..5] == "list<" && &typename[typename.len()-1..] == ">" => {
                Type::List(
                    Box::new(
                        Type::from_text_helper(&typename[5..typename.len()-1])?
                    )
                )
            },
            _ => {
                return None;
            }
        })
    }
    pub fn is_not(type0: &Type, type1: &Type) -> bool {
        match type0 {
            &Type::String => { if let &Type::String   = type1 { false } else { true }},
            &Type::Int    => { if let &Type::Int      = type1 { false } else { true }},
            &Type::Real   => { if let &Type::Real     = type1 { false } else { true }},
            &Type::Boole  => { if let &Type::Boole    = type1 { false } else { true }},
            &Type::Void   => { if let &Type::Void     = type1 { false } else { true }}
            &Type::List(ref t0) => { if let &Type::List(ref t1) = type1 { Type::is_not(t0, t1) } else { true }}
        }
    }
    pub fn check_signature(signature0: &[Type], signature1: &[Type]) -> bool {
        if signature0.len() != signature1.len() {
            return false;
        }
        for (type0, type1) in signature0.iter().zip(signature1.iter()) {
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
            Type::String         => write!(f, "string"),
            Type::Int            => write!(f, "int"),
            Type::Real           => write!(f, "real"),
            Type::Boole          => write!(f, "boole"),
            Type::Void           => write!(f, "void"),
            Type::List(ref cont) => write!(f, "list<{}>", cont),
        }
    }
}

impl Sexpr {
    fn type_check(&mut self, source: &str, all_scopes: &Vec<Scope>, function_manager: &mut FunctionManager) -> Result<Type, Error> {
        let ret_type: Type =
            match &mut *self.kind {
                SexprKind::Declare {ref declare_type, variable_name: _, ref mut expr, ref mut body} => {
                    let expr_type = expr.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(declare_type, &expr_type) {
                        return Err(Error::new(format!("initializing a variable of type {} to an expression of type {}", declare_type, expr_type), expr.token));
                    }
                    body.type_check(source, all_scopes, function_manager)?
                },
                SexprKind::Assign {ref variable_name, ref mut expr} => {
                    let var_type = all_scopes[self.scope.unwrap()].lookup_variable_type(all_scopes, &variable_name).unwrap();
                    let expr_type = expr.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(&var_type, &expr_type) {
                        return Err(Error::new(format!("assigning an expression of type {} to variable of type {}", expr_type, var_type), self.token));
                    }
                    var_type
                }
                SexprKind::IfSwitch {ref mut predicate, ref mut if_branch, ref mut else_branch} => {
                    let predicate_type = predicate.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(&predicate_type, &Type::Boole) {
                        return Err(Error::new(format!("if condition must be of type boole not {}", predicate_type), self.token));
                    }
                    let if_type   =   if_branch.type_check(source, all_scopes, function_manager)?;
                    let else_type = else_branch.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(&if_type, &else_type) {
                        return Err(Error::new_many(format!("branches must be of the same type. {} is not {}", if_type, else_type), vec![if_branch.token, else_branch.token]));
                    }
                    if_type
                }
                SexprKind::WhileLoop {ref mut predicate, ref mut body} => {
                    let predicate_type = predicate.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(&predicate_type, &Type::Boole) {
                        return Err(Error::new(format!("while condition must be of type boole, not {}", predicate_type), self.token));
                    }
                    body.type_check(source, all_scopes, function_manager)?
                }
                SexprKind::Block {ref mut statements} => {
                    let mut statement_type = Type::Void;
                    for i in 0..statements.len() {
                        statement_type = statements[i].type_check(source, all_scopes, function_manager)?;
                    }
                    statement_type
                }
                SexprKind::List {ref mut elements} => {
                    let mut inner_type: Option<Type> = None;
                    for ref mut elem in elements {
                        let elem_type = elem.type_check(source, all_scopes, function_manager)?;
                        if let Some(desired_type) = inner_type {
                            if Type::is_not(&elem_type, &desired_type) {
                                return Err(Error::new(format!("list element of type {} is incompatible with list<{}>", elem_type, desired_type), elem.token))
                            }
                        }
                        inner_type = Some(elem_type);
                    }
                    //TODO implement empty lists: how to deal with type-checking? use generic types? NOTE: basic does not let you go {} to make empty lists
                    let inner_type = inner_type.ok_or(Error::new("empty lists are not implemented yet".to_string(), self.token))?;
                    Type::List(Box::new(inner_type))
                }
                SexprKind::ListGet{ref mut list, ref mut index} => {
                    let index_type = index.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(&index_type, &Type::Int) {
                        return Err(Error::new(format!("list indices must be of type int, not {}", index_type), index.token));
                    }
                    let list_type = list.type_check(source, all_scopes, function_manager)?;
                    if let Type::List(ref inner_type) = list_type {
                        *inner_type.clone()
                    } else {
                        return Err(Error::new(format!("can not access indices of type {}", list_type), list.token));
                    }
                }
                SexprKind::ListSet{ref mut list, ref mut index, ref mut elem} => {
                    let index_type = index.type_check(source, all_scopes, function_manager)?;
                    if Type::is_not(&index_type, &Type::Int) {
                        return Err(Error::new(format!("list indices must be of type int, not {}", index_type), index.token));
                    }
                    let list_type = list.type_check(source, all_scopes, function_manager)?;
                    if let Type::List(ref inner_type) = list_type {
                        let elem_type = elem.type_check(source, all_scopes, function_manager)?;
                        if Type::is_not(&inner_type, &elem_type) {
                            return Err(Error::new(format!("assigning expression of type {} to elements from list of type {}", list_type, elem_type), elem.token))
                        }
                        Type::Void
                    } else {
                        return Err(Error::new(format!("can not access indices of non-indexable type {}", list_type), list.token));
                    }
                }
                SexprKind::FunctionDefinition { ref id } => {
                    // make sure that the function's body matches up with the out type
                    let closure =
                        |body: &mut Sexpr, manager: &mut FunctionManager| body.type_check(source, all_scopes, manager);
                    let body_type = function_manager.apply_self_mut(*id, closure)?;

                    if Type::is_not(&body_type, &function_manager.func_out_type[*id]) {
                        return Err(Error::new(format!("function body returns {} but function declaration states {}", body_type, function_manager.func_out_type[*id]), self.token));
                    }
                    Type::Void
                },
                SexprKind::Other { ref name, ref mut kind, ref mut exprs } => {
                    use sexprizer::OtherKind;
                    let mut types = vec![];
                    for ref mut expr in exprs {
                        types.push( expr.type_check(source, all_scopes, function_manager)? );
                    }
                    if let Some(func_id) = all_scopes[self.scope.unwrap()].resolve_function(all_scopes, name, &types) {
                        let call_id = function_manager.declare_call_site(func_id)?;
                        *kind = OtherKind::FuncCall{func_id, call_id};
                        function_manager.func_out_type[func_id].clone()
                    } else if let Some(new_id) = lang_consts::get_id(name, &types) {
                        *kind = OtherKind::BuiltIn{ id: new_id };
                        lang_consts::return_type_from_id(new_id)
                    } else {
                        return Err(Error::new(format!("no function found with name: `{}` and type signature: {:?}", name, types), self.token))
                    }
                }
                SexprKind::Identifier       => all_scopes[self.scope.unwrap()].lookup_variable_type(all_scopes, self.token.get_text(source)).unwrap(),
                SexprKind::RealLiteral      => Type::Real,
                SexprKind::IntegerLiteral   => Type::Int,
                SexprKind::StringLiteral    => Type::String,
                SexprKind::BooleLiteral     => Type::Boole,
        };
        self.return_type = Some(ret_type.clone());
        Ok(ret_type)
    }
}

pub fn type_check_all(source: &str, all_scopes: &Vec<Scope>, sexprs: &mut Vec<Sexpr>, function_manager: &mut FunctionManager) -> Result<(), Error> {
    for ref mut sexpr in sexprs {
        sexpr.type_check(source, all_scopes, function_manager)?;
    }
    Ok(())
}