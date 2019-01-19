use sexprizer::*;
use type_checker::Type;
use functionizer::FunctionManager;
use util::Error;
use std::collections::HashMap;
use variablizer::ValRepr;

#[derive(Debug)]
pub struct Scope {
    //map the names of variables to variable ids
    declared_variables: HashMap<String, usize>,
    //indexed by variable ids
    variable_types: Vec<Type>,
    //indexed by variable ids
    pub bound_reprs: Vec<ValRepr>, // variable reprs that have names in the source program
    bound_count: usize,
    pub unbound_count: usize,
    // store the name, type signature, and id of every function in scope
    declared_functions: Vec<(String, Vec<Type>, usize)>,
    pub is_func_def: bool,
    // None iff this is the global scope
    pub parent_index: Option<usize>,
    // empty for global scopes
    pub child_indices: Vec<usize>,
}
impl Scope {
    ///create a new scope with the given parent index and parent variable_count
    fn new(parent: Option<usize>, is_func_def: bool) -> Scope {
        Scope{
            declared_variables: HashMap::new(),
            variable_types: vec![],
            bound_reprs: vec![],
            bound_count: 0,
            unbound_count: 0,
            declared_functions: vec![],
            is_func_def,
            parent_index: parent,
            child_indices: vec![],
        }
    }
    ///create a new scope, adding it to the vector of scopes. return the index of the child scope
    fn create_child_to_vec(all_scopes: &mut Vec<Scope>, parent_index: usize) -> usize {
        let scope = Scope::new(Some(parent_index), false);
        all_scopes.push(scope);
        let index = all_scopes.len() - 1;
        all_scopes[parent_index].child_indices.push(index);
        index
    }
    fn create_global_to_vec(all_scopes: &mut Vec<Scope>) -> usize {
        let scope = Scope::new(None, false);
        all_scopes.push(scope);
        all_scopes.len() - 1
    }
    fn create_func_def_to_vec(all_scopes: &mut Vec<Scope>) -> usize {
        let scope = Scope::new(None, true);
        all_scopes.push(scope);
        all_scopes.len() - 1
    }
    pub fn count_total_vars(&self, all_scopes: &Vec<Scope>) -> usize {
        let total = self.bound_count + self.unbound_count;
        if let Some(child_total) = self.child_indices
            .iter()
            .map(|&index| all_scopes[index].count_total_vars(all_scopes))
            .max() {
            if child_total > total {
                child_total
            } else {
                total
            }
        } else {
            total
        }

    }
    pub fn add_func_binding(&mut self, name: String, signature: Vec<Type>, id: usize) {
        self.declared_functions.push((name, signature, id));
    }
    pub fn resolve_function(&self, all_scopes: &Vec<Scope>, name: &str, signature: &[Type]) -> Option<usize> {
        for (ref fname, ref fsignature, ref id) in self.declared_functions.iter().rev() {
            if fname == name && Type::check_signature(fsignature, signature) {
                return Some(*id);
            }
        }
        if let Some(index) = self.parent_index {
            all_scopes[index].resolve_function(all_scopes, name, signature)
        } else {
            None
        }
    }
    pub fn add_variable_binding(&mut self, name: String, variable_type: &Type) {
        self.declared_variables.insert(name, self.bound_count);
        self.variable_types.push(variable_type.clone());
        self.bound_count += 1;
    }
    pub fn lookup_variable_id(&self, all_scopes: &Vec<Scope>, name: &str) -> Option<usize> {
        // if we can't get the name, check if we have a parent. if we do, then ask them
        if let Some(id) = self.declared_variables.get(name) {
            Some(*id)
        } else {
            if let Some(index) = self.parent_index {
                all_scopes[index].lookup_variable_id(all_scopes, name)
            } else {
                None
            }
        }
    }
    pub fn lookup_variable_type(&self, all_scopes: &Vec<Scope>, name: &str) -> Option<Type> {
        if let Some(id) = self.lookup_variable_id(all_scopes, name) {
            Some(self.variable_types[id].clone())
        } else {
            None
        }
    }
    pub fn lookup_val_repr(&self, all_scopes: &Vec<Scope>, name: &str) -> Option<ValRepr> {

        if let Some(id) = self.lookup_variable_id(all_scopes, name) {
            //println!("{:#?}", all_scopes);
            //WARN we assume that we are the scope where the repr is bound: we should look recursively
            Some(self.bound_reprs[id].clone())
        } else {
            None
        }
    }
}

fn bind_variable(all_scopes: &mut Vec<Scope>, index: usize, name: &String, _type: &Type) {
    all_scopes[index].add_variable_binding(name.clone(), _type); //TODO shadow warning?
}

impl Sexpr {
    ///creates the scope for this sexpr and all its children, returning ownership on a success.
    fn create_scope(&mut self, source: &str, all_scopes: &mut Vec<Scope>, parent_index: usize, function_manager: &mut FunctionManager) -> Result<(), Error> {
        self.scope = Some(parent_index);
        use std::ops::DerefMut;
        match &mut self.kind.deref_mut() {
            SexprKind::Declare {ref declare_type, ref variable_name, ref mut expr, ref mut body} => {
                // create a child scope with the variable_name : declare_type binding
                let child_index = Scope::create_child_to_vec(all_scopes, parent_index);
                bind_variable(all_scopes, child_index, variable_name, declare_type);
                expr.create_scope(source, all_scopes, parent_index, function_manager)?;
                body.create_scope(source, all_scopes, child_index, function_manager)?;
            },
            SexprKind::Assign {ref variable_name, ref mut expr} => {
                if all_scopes[parent_index].lookup_variable_type(all_scopes, &variable_name).is_none() {
                    return Err(Error::new(format!("assigning to undeclared variable `{}`", variable_name), self.token));
                }
                expr.create_scope(source, all_scopes, parent_index, function_manager)?;
            }
            SexprKind::IfSwitch {ref mut predicate, ref mut if_branch, ref mut else_branch} => {
                predicate.create_scope(source, all_scopes, parent_index, function_manager)?;
                if_branch.create_scope(source, all_scopes, parent_index, function_manager)?;
                else_branch.create_scope(source, all_scopes, parent_index, function_manager)?;
            }
            SexprKind::WhileLoop {ref mut predicate, ref mut body} => {
                predicate.create_scope(source, all_scopes, parent_index, function_manager)?;
                body.create_scope(source, all_scopes, parent_index, function_manager)?;
            }
            SexprKind::Block {ref mut statements} => {
                for ref mut statement in statements {
                    statement.create_scope(source, all_scopes, parent_index, function_manager)?;
                }
            }
            SexprKind::List { ref mut elements } => {
                for ref mut elem in elements {
                    elem.create_scope(source, all_scopes, parent_index, function_manager)?;
                }
            }
            SexprKind::ListGet { ref mut list, ref mut index} => {
                list.create_scope(source, all_scopes, parent_index, function_manager)?;
                index.create_scope(source, all_scopes, parent_index, function_manager)?;
            }
            SexprKind::ListSet { ref mut list, ref mut index, ref mut elem} => {
                list.create_scope(source, all_scopes, parent_index, function_manager)?;
                index.create_scope(source, all_scopes, parent_index, function_manager)?;
                elem.create_scope(source, all_scopes, parent_index, function_manager)?;
            }
            SexprKind::FunctionDefinition { ref id } => {
                let id = *id;
                all_scopes[parent_index].add_func_binding(function_manager.func_name[id].clone(), function_manager.func_signature[id].clone(), id);
                // create our own parallel scoping business
                let new_index = Scope::create_func_def_to_vec(all_scopes);
                all_scopes[new_index].add_func_binding(function_manager.func_name[id].clone(), function_manager.func_signature[id].clone(), id);
                for i in 0..(function_manager.func_args[id].len()) {
                    all_scopes[new_index].add_variable_binding(function_manager.func_args[id][i].clone(), &function_manager.func_signature[id][i]);
                }
                function_manager.apply_self_mut(id, |body: &mut Sexpr, manager: &mut FunctionManager| body.create_scope(source, all_scopes, new_index, manager))?;
            },
            SexprKind::Other { name: _, kind: _, ref mut exprs } => {
                for ref mut expr in exprs {
                    expr.create_scope(source, all_scopes, parent_index, function_manager)?;
                }
            }
            SexprKind::Identifier => {
                if all_scopes[parent_index].lookup_variable_type(all_scopes, self.token.get_text(source)).is_none() {
                    return Err(Error::new(format!("variable is undeclared"), self.token));
                }
            }
            SexprKind::RealLiteral | SexprKind::IntegerLiteral | SexprKind::StringLiteral | SexprKind::BooleLiteral => {},
        }
        Ok(())
    }
}

pub fn create_all_scopes(source: &str, sexprs: &mut Vec<Sexpr>, function_manager: &mut FunctionManager) -> Result<Vec<Scope>, Error> {
    let mut all_scopes = vec![];
    Scope::create_global_to_vec(&mut all_scopes);
    for ref mut sexpr in sexprs {
        sexpr.create_scope(source, &mut all_scopes, 0, function_manager)?;
    }
    Ok(all_scopes)
}