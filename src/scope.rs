use type_checker::Type;

use std::collections::{HashMap};


#[derive(Debug, Copy, Clone)]
pub struct ScopeId { pub index: usize }

#[derive(Debug)]
pub struct Scope {
    //map the names of variables to variable ids
    pub declared_variables: HashMap<String, usize>,
    //indexed by variable ids
    pub variable_types: Vec<Option<Type>>,

    // store the id of every function in scope
    pub declared_functions: Vec<usize>,
    pub is_func_def: bool,
    // store the  id of every struct in scope
    pub declared_structs: Vec<usize>,
    // None iff this is the global scope
    pub parent: Option<ScopeId>,
    // empty for global scopes
    pub children: Vec<ScopeId>,
}
impl Scope {
    ///create a new scope, adding it the vector of all scopes
    pub fn new(parent: Option<ScopeId>, is_func_def: bool) -> Scope {
        Scope{
            declared_variables: HashMap::new(),
            variable_types: vec![],
            declared_functions: vec![],
            is_func_def,
            declared_structs: vec![],
            parent,
            children: vec![],
        }
    }
    pub fn new_to_vec(all_scopes: &mut Vec<Scope>, parent: Option<ScopeId>, is_func_def: bool) -> ScopeId {
        all_scopes.push(Scope::new(parent, is_func_def));
        ((all_scopes.len() - 1) as usize).into()
    }
}

impl std::convert::From<usize> for ScopeId {
    fn from(index: usize) -> ScopeId {
        ScopeId { index }
    }
}