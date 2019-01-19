use sexprizer::{Sexpr, SexprKind};
use scoper::Scope;
use type_checker::Type;
use util::Error;
use builder::BuildFlags;
use functionizer::FunctionManager;

#[derive(Debug, Clone)]
pub enum ValRepr {
    /// for compile-time fictions such as void types: no value can be read from or written to us
    ZeroSized,
    /// regular variables: A, B, Str2, ⌊LIST0
    Simple(String),
    /// a complex number a+bi where a,b tells us the substring of Str0
    IndexString(String),
}
impl ValRepr {
    pub fn new_void() -> ValRepr {
        ValRepr::ZeroSized
    }
    /// read this value
    /// returns a valid TI-84 Basic expression
    pub fn read(&self) -> String {
        match *self {
            ValRepr::ZeroSized => "".to_owned(),
            ValRepr::Simple(ref s) => s.clone(),
            ValRepr::IndexString(ref s) => format!("substring(Str0, real({}), imag({}))", s, s),
        }
    }
    /// return a version of this value that's numeric,
    /// edit the program string and build flag if any transmutation is required
    /// used for writing to the inside of lists
    pub fn transmute_num(&mut self, val_type: &Type, prgm: &mut String, build_flags: &mut BuildFlags) -> Option<String> {
        Some(match val_type {
            Type::Int | Type::Real | Type::Boole => self.read(), // we're already numeric: do nothing but read as normal
            Type::Void | Type::List(_) => {
                // we have no numeric representation //TODO handle void somehow within lists?
                return None;
            },
            Type::String => {
                match self {
                    ValRepr::ZeroSized => panic!("strings are not zero-sized. somehow one was created"),
                    ValRepr::Simple(ref s) => {
                        // s refers to the string which we would like to transmute
                        // append s to the master string
                        build_flags.initialize_master_string = true;
                        prgm.push_str(&format!("Str0+{}→Str0\n", s));
                        // write the beginning of the string into the real part, and the end into the imaginary
                        // assumes that no one has altered the master string
                        format!("dim(Str0)-length({})+idim(Str0)", s)
                    }
                    ValRepr::IndexString(ref s) => s.clone(),
                }
            }
        })
    }

    /// interpret a given string (which we are told was transmuted into a numeric value)
    /// wrap it in a corresponding ValRepr
    /// used for returning from the inside of lists
    pub fn interpret_num(handle: String, val_type: &Type) -> ValRepr {
        match val_type {
            &Type::String => ValRepr::IndexString(handle),
            _ => ValRepr::Simple(handle),
        }
    }

    /// write to this value
    /// returns code that does the assignment
    pub fn write(&self, prgm: &mut String, value: &ValRepr) {
        // prevent spurious assignment, i.e.
        //   A->A
        // and assigning to void types
        let value_text: String = value.read();
        if self.read().len() == 0 || value_text.len() == 0 || self.read() == value.read() {
            //WARN we hide away statements here because we didn't want to write an expression to itself
            return;
        }
        match *self {
            ValRepr::ZeroSized => {}
            ValRepr::Simple(ref s) => {
                prgm.push_str(&format!("{}→{}\n", value_text, s));
            },
            ValRepr::IndexString(ref s) => {
                prgm.push_str(&format!("dim(Str0)→{s}\nStr0+{value}→{s}\n{s}+idim(Str0)→{s}\n", s = s, value = value_text));
            },
        }
    }
}

#[derive(Debug)]
pub struct VariableManager {
    numeric_counter: usize,
    string_counter: usize,
    list_counter: usize,
    func_arg_counter: usize,
    in_func_def: bool,
}
impl VariableManager {
    pub fn new() -> VariableManager {
        VariableManager {
            numeric_counter: 0,
            string_counter: 1, // begin at 1 to allow master string
            list_counter: 0,
            func_arg_counter: 1, // leave the zero empty for the call-back
            in_func_def: false,
        }
    }

    // TODO numeric - list splicing
    fn create_numeric_tag(&mut self) -> Result<String, Error> {
        let num = self.numeric_counter;
        self.numeric_counter += 1;
        if num < 26 {
            Ok("ABCDEFGHIJKLMNOPQRSTUVWXYZΘ"[num..num+1].to_string())
        } else {
            Err(Error::new_zero(format!("exceeded numeric variable maximum (at maximum: 26 distinct numeric types. number sharing, list representatios not implemented")))
        }
    }
    fn create_string_tag(&mut self) -> Result<String, Error> {
        let num = self.string_counter;
        self.string_counter += 1;
        if num < 10 {
            Ok(format!("Str{}", num))
        } else {
            Err(Error::new_zero(format!("exceeded string variable maximum (at maximum: 10 distinct strings. string slicing not implemented)")))
        }
    }
    fn create_list_tag(&mut self) -> String {
        let num = self.list_counter;
        self.list_counter += 1;
        format!("⌊LIST{}", num)
    }
    fn create_in_func_tag(&mut self) -> String {
        let num = self.func_arg_counter;
        self.func_arg_counter += 1;
        format!("⌊ARGS(dim(⌊ARGS)-{})", num)
    }

    /// create a tag, assuming that we are never expected to cough up that same tag somewhere else
    /// used for temporarily storing values, such as the results from either branch of an if-switch
    /// MUST BE CALLED IN EXECUTION ORDER
    fn make_repr(&mut self, var_type: &Type) -> Result<ValRepr, Error> {
        if self.in_func_def {
            //println!("inside a function definition!\n{:?}", self);
            return Ok(ValRepr::interpret_num(self.create_in_func_tag(), var_type));
        }
        Ok(match var_type {
            &Type::String => ValRepr::Simple(self.create_string_tag()?),
            &Type::Int | &Type::Real | &Type::Boole => ValRepr::Simple(self.create_numeric_tag()?),
            &Type::List(_) => ValRepr::Simple(self.create_list_tag()),
            &Type::Void => ValRepr::Simple(String::new())
        })
    }

    /// associate each variable name with a unique value to use in building
    /// WARNING: the parent's variable tags must be finalized BEFORE calling the child
    pub fn make_bound_repr(&mut self, all_scopes: &mut Vec<Scope>, scope_index: usize, name: &str) -> Result<(), Error> {
        let var_id = all_scopes[scope_index].lookup_variable_id(all_scopes, name).unwrap(); // we can be sure that this name has been bound already
        assert_eq!(all_scopes[scope_index].bound_reprs.len(), var_id); // we must add only to the end of the vector
        let var_type = all_scopes[scope_index].lookup_variable_type(all_scopes, name).unwrap();
        let repr = self.make_repr(&var_type)?;
        all_scopes[scope_index].bound_reprs.push(repr);
        Ok(())
    }
    /// associate each variable name with a unique value to use in building
    /// WARNING: the parent's variable tags must be finalized BEFORE calling the child
    pub fn make_free_repr(&mut self, all_scopes: &mut Vec<Scope>, scope_index: usize, var_type: &Type, associated_reprs: &mut Vec<ValRepr>) -> Result<(), Error> {
        let repr = self.make_repr(var_type)?;
        all_scopes[scope_index].unbound_count += 1;
        associated_reprs.push(repr);
        Ok(())
    }

    /// must be called before a function definition
    /// call exit_func_def after exiting
    pub fn enter_func_def(&mut self) {
        self.in_func_def = true;
    }
    /// must be called after a function definition
    /// call enter_func_def before exiting
    pub fn exit_func_def(&mut self) {
        self.in_func_def = false;
    }
}

impl Sexpr {
    ///Create all the variables which are used only internally (created by ifSwitch and the like)
    fn create_variables(&mut self, source: &str, all_scopes: &mut Vec<Scope>, var_manager: &mut VariableManager, manager: &mut FunctionManager) -> Result<(), Error> {
        match &mut *self.kind {
            SexprKind::Declare { declare_type:_ , ref variable_name, ref mut expr, ref mut body } => {
                expr.create_variables(source, all_scopes, var_manager, manager)?;
                var_manager.make_bound_repr(all_scopes,
                                            body.scope.unwrap(),
                                            variable_name)?;
                body.create_variables(source, all_scopes, var_manager, manager)?;
            },
            SexprKind::Assign { variable_name: _, ref mut expr } => {
                expr.create_variables(source, all_scopes, var_manager, manager)?;
            },
            SexprKind::IfSwitch {ref mut predicate, ref mut if_branch, ref mut else_branch} => {
                predicate.create_variables(source, all_scopes, var_manager, manager)?;
                var_manager.make_free_repr(all_scopes,
                                           self.scope.unwrap(),
                                           self.return_type.as_ref().unwrap(),
                                           &mut self.associated_reprs)?;
                if_branch.create_variables(source, all_scopes, var_manager, manager)?;
                else_branch.create_variables(source, all_scopes, var_manager, manager)?;
            },
            SexprKind::WhileLoop { ref mut predicate, ref mut body } => {
                predicate.create_variables(source, all_scopes, var_manager, manager)?;
                body.create_variables(source, all_scopes, var_manager, manager)?;
            },
            SexprKind::List { ref mut elements } => {
                var_manager.make_free_repr(all_scopes,
                                           self.scope.unwrap(),
                                           self.return_type.as_ref().unwrap(),
                                           &mut self.associated_reprs)?;
                for ref mut elem in elements {
                    elem.create_variables(source, all_scopes, var_manager, manager)?;
                }
            }
            SexprKind::ListGet { ref mut index, ref mut list } => {
                list.create_variables(source, all_scopes, var_manager, manager)?;
                index.create_variables(source, all_scopes, var_manager, manager)?;
            }
            SexprKind::ListSet { ref mut index, ref mut list, ref mut elem } => {
                list.create_variables(source, all_scopes, var_manager, manager)?;
                index.create_variables(source, all_scopes, var_manager, manager)?;
                elem.create_variables(source, all_scopes, var_manager, manager)?;
            }
            SexprKind::Block { ref mut statements } => {
                for ref mut expr in statements {
                    expr.create_variables(source, all_scopes, var_manager, manager)?;
                }
            },
            SexprKind::FunctionDefinition { ref id } => {
                var_manager.enter_func_def();

                let body_index = manager.func_body[*id].scope.unwrap();
                for ref name in manager.func_args[*id].iter() {
                    var_manager.make_bound_repr(all_scopes, body_index, name)?;
                }
                manager.apply_self_mut(*id,
                                       |body: &mut Sexpr, manager: &mut FunctionManager| {
                                           body.create_variables(source, all_scopes, var_manager, manager)
                                       })?;

                var_manager.exit_func_def();
            }
            SexprKind::Other { name: _, kind: _, ref mut exprs } => {
                // we need no variables as of yet; only what our arguments need
                for ref mut expr in exprs {
                    expr.create_variables(source, all_scopes, var_manager, manager)?;
                }
            }
            SexprKind::Identifier | SexprKind::BooleLiteral | SexprKind::RealLiteral | SexprKind::IntegerLiteral | SexprKind::StringLiteral => { },
        }
        Ok(())
    }
}

pub fn create_all_variables(source: &str, all_scopes: &mut Vec<Scope>, sexprs: &mut Vec<Sexpr>, manager: &mut FunctionManager) -> Result<(), Error> {
    let mut var_manager = VariableManager::new();
    for ref mut sexpr in sexprs {
        sexpr.create_variables(source, all_scopes, &mut var_manager, manager)?;
    }
    Ok(())
}




