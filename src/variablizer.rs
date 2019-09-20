use sexpr::{SexprId};
use scope::ScopeId;
use type_checker::Type;
use util::Error;
use builder::BuildFlags;
use manager::Manager;

fn encode_pair(left: &str, right: &str) -> String {
    format!("{}+{}/9", left, right) // add them together, dividing the right by 10^9
}
fn decode_pair(expr: &str) -> (String, String) {
    (format!("iPart({})", expr), format!("9fPart({})", expr))
}

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
    /// get this value's tag
    /// panics if it is a repr that does not have a tag
    /// the tag may not match the repr's compile time type
    pub fn get_tag(&self) -> &str {
        match *self {
            ValRepr::ZeroSized => panic!("zero-sized types have no tag"),
            ValRepr::Simple(ref s) => s,
            ValRepr::IndexString(ref s) => s,
        }
    }
    /// read this value
    /// returns a valid TI-84 Basic expression
    pub fn read(&self) -> String {
        match *self {
            ValRepr::ZeroSized => "".to_owned(),
            ValRepr::Simple(ref s) => s.to_owned(),
            ValRepr::IndexString(ref s) => {
                let (l, r) = decode_pair(s);
                format!("sub(Str0,{},{})", l, r)
            },
        }
    }
    /// return a version of this value that's numeric,
    /// edit the program string and build flag if any transmutation is required
    /// used for writing to the inside of lists
    pub fn transmute_num(&self, val_type: &Type, prgm: &mut String, build_flags: &mut BuildFlags) -> Option<String> {
        Some(match val_type {
            Type::Int | Type::Real | Type::Boole | Type::CustomType(_, _) => self.read(), // we're already numeric: do nothing but read as normal
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
                        encode_pair(
                            &format!("1+length(Str0)-length({})", s),
                            &format!("length({})", s))
                    }
                    ValRepr::IndexString(ref s) => s.to_owned(),
                }
            }
        })
    }

    /// interpret a given string (which we are told was transmuted into a numeric value)
    /// wrap it in a corresponding ValRepr
    /// used for returning from the inside of lists
    pub fn interpret_num(handle: String, val_type: &Type) -> ValRepr {
        if val_type.is_void() {
            return ValRepr::new_void();
        }
        match val_type {
            &Type::String => ValRepr::IndexString(handle),
            _ => ValRepr::Simple(handle),
        }
    }

    /// write to self, so that it becomes equal to `value`
    /// returns code that does the assignment
    pub fn write(&self, prgm: &mut String, value: &ValRepr) {
        // prevent spurious assignment, i.e.
        //   A->A
        // and assigning to void types
        let value_text = value.read();
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
                if let ValRepr::IndexString(ref r) = value {
                    prgm.push_str(&format!("{}→{}\n", r, s));
                } else {
                    prgm.push_str(&format!("length(Str0)→{s}\nStr0+{value}→{s}\n{s}+length(Str0)/9→{s}\n", s = s, value = value_text));
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct VariableAssigner {
    numeric_counter: usize,
    string_counter: usize,
    list_counter: usize,
    func_arg_counter: usize,
    in_func_def: bool,
}
impl VariableAssigner {
    pub fn new() -> VariableAssigner {
        VariableAssigner {
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
    fn make_repr(&mut self, var_type: &Type, numeric: bool) -> Result<ValRepr, Error> {
        if self.in_func_def {
            //println!("inside a function definition!\n{:?}", self);
            return Ok(ValRepr::interpret_num(self.create_in_func_tag(), var_type));
        }
        Ok(match var_type {
            &Type::String => {
                if numeric {
                    ValRepr::IndexString(self.create_numeric_tag()?)
                } else {
                    ValRepr::Simple(self.create_string_tag()?)
                }
            },
            &Type::Int | &Type::Real | &Type::Boole | &Type::CustomType(_, _) => ValRepr::Simple(self.create_numeric_tag()?),
            &Type::List(_) => {
                if numeric {
                    panic!("no numeric list represnetations yet")
                } else {
                    ValRepr::Simple(self.create_list_tag())
                }
            },
            &Type::Void => ValRepr::Simple(String::new())
        })
    }

    /// associate each variable name with a unique value to use in building
    /// WARNING: the parent's variable tags must be finalized BEFORE calling the child
    pub fn make_bound_repr(&mut self, m: &mut Manager, scope: ScopeId, name: &str) -> Result<(), Error> {
        scope.inform_var_repr(m, name, |var_type| self.make_repr(var_type, false))
    }
    /// make a variable for internal use by a s-expr
    pub fn make_free_repr(&mut self, m: &mut Manager, scope: ScopeId, var_type: &Type, associated_reprs: &mut Vec<ValRepr>) -> Result<(), Error> {
        let repr = self.make_repr(var_type, false)?;
        scope.scope_mut(m).unbound_count += 1;
        associated_reprs.push(repr);
        Ok(())
    }
    /// make a variable for internal use by a s-expr that is numeric
    pub fn make_free_numeric_repr(&mut self, m: &mut Manager, scope: ScopeId, var_type: &Type, associated_reprs: &mut Vec<ValRepr>) -> Result<(), Error> {
        let repr = self.make_repr(var_type, true)?;
        scope.scope_mut(m).unbound_count += 1;
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

impl SexprId {
    ///Create all the variables both internally (created by ifSwitch and the like) and variables declared by user
    fn create_variables(self, m: &mut Manager, var_assigner: &mut VariableAssigner) -> Result<(), Error> {
       /* match &mut *self.kind {
            SexprKind::Declare { ref variable_pattern, ref mut expr, ref mut body } => {
                expr.create_variables(m, var_assigner)?;
                var_assigner.make_bound_repr(m, body.my_scope(m), variable_pattern)?;
                body.create_variables(m, var_assigner)?;
            },
            SexprKind::Assign { variable_pattern: _, ref mut expr } => {
                expr.create_variables(m, var_assigner)?;
            },
            SexprKind::IfSwitch {ref mut predicate, ref mut if_branch, ref mut else_branch} => {
                predicate.create_variables(m, var_assigner)?;
                var_assigner.make_free_repr(all_scopes,
                                           self.scope.unwrap(),
                                           self.return_type.as_ref().unwrap(),
                                           &mut self.associated_reprs)?;
                if_branch.create_variables(m, var_assigner)?;
                else_branch.create_variables(m, var_assigner)?;
            },
            SexprKind::WhileLoop { ref mut predicate, ref mut body } => {
                predicate.create_variables(m, var_assigner)?;
                body.create_variables(m, var_assigner)?;
            },
            SexprKind::List { ref mut elements } => {
                var_assigner.make_free_repr(all_scopes,
                                           self.scope.unwrap(),
                                           self.return_type.as_ref().unwrap(),
                                           &mut self.associated_reprs)?;
                for ref mut elem in elements {
                    elem.create_variables(m, var_assigner)?;
                }
            }
            SexprKind::ListGet { ref mut index, ref mut list } => {
                list.create_variables(m, var_assigner)?;
                index.create_variables(m, var_assigner)?;
            }
            SexprKind::ListSet { ref mut index, ref mut list, ref mut elem } => {
                list.create_variables(m, var_assigner)?;
                index.create_variables(m, var_assigner)?;
                elem.create_variables(m, var_assigner)?;
            }
            SexprKind::Block { ref mut statements } => {
                for ref mut expr in statements {
                    expr.create_variables(m, var_assigner)?;
                }
            },
            SexprKind::FuncDef { ref id } => {
                var_assigner.enter_func_def();

                let body_index = m.func_manager.get_func_body_ref(*id).scope.unwrap();
                for ref name in m.func_manager.func_args[*id].iter() {
                    var_assigner.make_bound_repr(all_scopes, body_index, name)?;
                }
                m.apply_self_mut(*id,
                                       |body: &mut Sexpr, manager: &mut Manager| {
                                           body.create_variables(source, all_scopes, var_assigner, manager)
                                       })?;

                var_assigner.exit_func_def();
            }
            SexprKind::FuncCall { func_id: _, call_id: _, ref mut exprs } => {
                var_assigner.make_free_repr(all_scopes,
                                           self.scope.unwrap(),
                                           self.return_type.as_ref().unwrap(),
                                           &mut self.associated_reprs)?;
                for ref mut expr in exprs {
                    expr.create_variables(m, var_assigner)?;
                }
            }
            SexprKind::StructDef { id: _ } => {},
            SexprKind::StructInit { id: _, ref mut exprs } => {
                var_assigner.make_free_repr(all_scopes,
                                           self.scope.unwrap(),
                                           self.return_type.as_ref().unwrap(),
                                           &mut self.associated_reprs)?;
                for ref mut expr in exprs {
                    expr.create_variables(m, var_assigner)?;
                }
            }
            SexprKind::StructGet { id:_, ref mut expr, field:_ } => {
                expr.create_variables(m, var_assigner)?;
            },
            SexprKind::StructSet { id:_, ref mut expr, field:_, ref mut value } => {
                expr.create_variables(m, var_assigner)?;
                value.create_variables(m, var_assigner)?;
            },
            SexprKind::Format { ref mut exprs } => {
                var_assigner.make_free_numeric_repr(all_scopes,
                                            self.scope.unwrap(),
                                            &Type::String,
                                            &mut self.associated_reprs)?;
                var_assigner.make_free_repr(all_scopes,
                                            self.scope.unwrap(),
                                            &Type::Real,
                                            &mut self.associated_reprs)?;
                for ref mut expr in exprs {
                    expr.create_variables(m, var_assigner)?;
                }
            },
            SexprKind::BuiltIn { id:_, ref mut exprs } => {
                for ref mut expr in exprs {
                    expr.create_variables(m, var_assigner)?;
                }
            },
            SexprKind::Other { opt_exprs: _ } => panic!("we should not be variablizing a SexprKind::Other"),
            SexprKind::Identifier | SexprKind::BooleLiteral | SexprKind::RealLiteral | SexprKind::IntegerLiteral | SexprKind::StringLiteral => { },
        }
        Ok(())
        */
        unimplemented!()
    }
}

pub fn create_all_variables(m: &mut Manager) -> Result<(), Error> {
    let mut var_assigner = VariableAssigner::new();
    for sexpr_id in m.top_level_sexprs {
        sexpr_id.create_variables(m, &mut var_assigner)?;
    }
    Ok(())
}




