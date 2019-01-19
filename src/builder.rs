use sexprizer::{Sexpr, SexprKind};
use scoper::Scope;
use lang_consts;
use functionizer::FunctionManager;
use variablizer::ValRepr;
use util::Error;
use util::vec_fmt;

pub struct BuildFlags {
    pub initialize_master_string: bool,
    pub initialize_stack_frames: bool,
}

impl BuildFlags {
    fn new() -> BuildFlags {
        BuildFlags {
            initialize_master_string: false,
            initialize_stack_frames: false,
        }
    }
}

fn get_variable(all_scopes: &Vec<Scope>, scope_index: usize, name: &str) -> ValRepr {
    all_scopes[scope_index].lookup_val_repr(all_scopes, name).expect(&format!("variablization  stage did not succeed in giving a label for variable of name {} in scope #{}", name, scope_index))
}

impl Sexpr {
    ///return a pair of `String`s: (handle, code)
    /// handle - what other pieces of code should use to refer to the result of this s-expr
    /// code - what other pieces of code need to run before refering to the result of this s-expr
    fn build(&self, prgm: &mut String, source: &str, all_scopes: &Vec<Scope>, function_manager: &FunctionManager, build_flags: &mut BuildFlags) -> Result<ValRepr, Error> {
        // generate the (handle, code) tuple
        //WARN the code must end in a newline
        Ok(match *self.kind {
            SexprKind::Declare { declare_type:_ , ref variable_name, ref expr, ref body } => {
                let variable = get_variable(all_scopes, body.scope.unwrap(), variable_name);
                let expr_handle = expr.build(prgm, source, all_scopes, function_manager, build_flags)?;
                variable.write(prgm, &expr_handle);
                body.build(prgm, source, all_scopes, function_manager, build_flags)?
            },
            SexprKind::Assign { ref variable_name, ref expr } => {
                let variable = get_variable(all_scopes, self.scope.unwrap(), variable_name);
                let expr_handle = expr.build(prgm, source, all_scopes, function_manager, build_flags)?;
                variable.write(prgm, &expr_handle);
                variable
            },
            SexprKind::IfSwitch {ref predicate, ref if_branch, ref else_branch} => {
                let variable = self.associated_reprs[0].clone();

                let predicate_repr = predicate.build(prgm, source, all_scopes, function_manager, build_flags)?;
                prgm.push_str(&format!("If {}\nThen\n", predicate_repr.read()));

                let if_repr = if_branch.build(prgm, source, all_scopes, function_manager, build_flags)?;
                variable.write(prgm, &if_repr);
                prgm.push_str("Else\n");

                let else_repr = else_branch.build(prgm, source, all_scopes, function_manager, build_flags)?;
                variable.write(prgm, &else_repr);
                prgm.push_str("End\n");

                variable
            },
            SexprKind::WhileLoop { ref predicate, ref body } => {
                let predicate_repr = predicate.build(prgm, source, all_scopes, function_manager, build_flags)?;
                prgm.push_str(&format!("While {}\n", predicate_repr.read()));
                let body_repr = body.build(prgm, source, all_scopes, function_manager, build_flags)?;
                prgm.push_str("End\n");
                body_repr
            },
            SexprKind::List { ref elements } => {
                //TODO the numeric version of this is much easier, simpler and does not require creating a variable
                let var = self.associated_reprs[0].clone();
                prgm.push_str(&format!("{len}→dim({var})\n", len = elements.len(), var = var.read()));
                for (index, elem) in elements.iter().enumerate() {
                    let mut elem_repr = elem.build(prgm, source, all_scopes, function_manager, build_flags)?;
                    // transmute the handle into a numeric form
                    let elem_handle = elem_repr
                        .transmute_num(elem.return_type.as_ref().unwrap(), prgm, build_flags)
                        .ok_or(Error::new(format!("Can not store value of type {} in a list (must have a valid numeric representation", elem.return_type.as_ref().unwrap()), elem.token))?;
                    // add to the list the new complex number. handle should be a variable.
                    prgm.push_str(&format!("{elem_handle}→{var}({index})\n",
                        elem_handle = elem_handle,
                        var = var.read(), //WARN this is something to consider, when we numericize lists: how to write to them / if they can be written to: how to throw the error?
                        index = index + 1, //TI -84 basic is 1 indexed
                    ));
                }
                var
            }
            SexprKind::ListGet { ref list, ref index } => {
                let list_repr = list.build(prgm, source, all_scopes, function_manager, build_flags)?;
                let index_repr = index.build(prgm, source, all_scopes, function_manager, build_flags)?;
                let handle = format!("{list}({i})", list = list_repr.read(), i = index_repr.read());
                ValRepr::interpret_num(handle, self.return_type.as_ref().unwrap())
            }
            SexprKind::ListSet { ref list, ref index, ref elem } => {
                let list_repr  = list.build(prgm, source, all_scopes, function_manager, build_flags)?;
                let index_repr = index.build(prgm, source, all_scopes, function_manager, build_flags)?;
                let elem_handle  = elem
                    .build(prgm, source, all_scopes, function_manager, build_flags)?
                    .transmute_num(elem.return_type.as_ref().unwrap(), prgm, build_flags)
                    .ok_or(Error::new(format!("Can not store value of type {} in a list (must have a valid numeric representation", elem.return_type.as_ref().unwrap()), elem.token))?;

                prgm.push_str(&format!("{elem_handle}→{list_repr}({index_repr})\n",
                            elem_handle = elem_handle,
                            list_repr = list_repr.read(),
                            index_repr = index_repr.read(),
                    ));
                ValRepr::new_void()
            }
            SexprKind::Block { ref statements } => {
                let mut repr = ValRepr::new_void();
                for ref statement in statements { // TODO mark unused statements as such <- why is this here? we need to do it in type-checking
                    repr = statement.build(prgm, source, all_scopes, function_manager, build_flags)?;
                }
                repr
            },
            SexprKind::FunctionDefinition { id: _ } => ValRepr::new_void(), // definitions have no in-place code
            SexprKind::Other { name: _, ref kind, ref exprs } => {
                use sexprizer::OtherKind;
                match kind {
                    OtherKind::Undecided => panic!("the compiler attempted to build {:#?} but it hasn't decided what it is yet", self),
                    OtherKind::BuiltIn { ref id } => {
                        let id = *id;
                        let mut val_reprs = vec![];
                        for ref expr in exprs {
                            val_reprs.push(expr.build(prgm, source, all_scopes, function_manager, build_flags)?);
                        }
                        let read_only_handles: Vec<String> = val_reprs.iter().map(|vr| vr.read()).collect();
                        let additional_code = vec_fmt(lang_consts::code_format_string_from_id(id), &read_only_handles);
                        prgm.push_str(&additional_code);
                        let my_handle = vec_fmt(lang_consts::handle_format_string_from_id(id), &read_only_handles);
                        ValRepr::Simple(my_handle)
                    },
                    OtherKind::FuncCall { ref func_id, ref call_id  } => {
                        // we want to push onto ⌊ARGS: everything in the scope of the function plus our call_id
                        let func_scope_index = function_manager.func_body[*func_id].scope.unwrap();
                        let var_total = all_scopes[func_scope_index].count_total_vars(all_scopes);
                        //println!("var_total - exprs.len() = {} - {}", var_total, exprs.len());
                        let local_count = var_total - exprs.len();
                        // initialize all the local variables
                        for _ in 0..local_count {
                            prgm.push_str("0→⌊ARGS(dim(⌊ARGS)+1)\n");
                        }
                        // initialize all the arguments
                        for ref expr in exprs {
                            let mut handle = expr
                                .build(prgm, source, all_scopes, function_manager, build_flags)?
                                .transmute_num(expr.return_type.as_ref().unwrap(), prgm, build_flags)
                                .ok_or(Error::new("Can not pass value of this type to function".to_owned(), expr.token))?; //TODO do this error handling in function definition
                            prgm.push_str(&format!("{}→⌊ARGS(dim(⌊ARGS)+1)\n", handle))
                        }
                        prgm.push_str(&format!("{call_id}→⌊ARGS(dim(⌊ARGS)+1)\nGoto {func_label}\nLbl {call_label}\ndim(⌊ARGS)-{size}→dim(⌊ARGS)\n",
                            call_id = *call_id,
                            func_label = function_manager.func_labels[*func_id],
                            call_label = function_manager.call_labels[*call_id],
                            size = var_total + 1, // all our variables (locals + args) + the call back
                        ));
                        if self.return_type.as_ref().unwrap().is_void() {
                            ValRepr::new_void()
                        } else {
                            ValRepr::Simple(format!("⌊RES(dim(⌊RES))"))
                        }
                    },
                }
            }
            SexprKind::Identifier => {
                get_variable(all_scopes, self.scope.unwrap(), self.token.get_text(source))
            },
            SexprKind::BooleLiteral => {
                ValRepr::Simple(match self.token.get_text(source) {
                    "true"  => String::from("1"),
                    "false" => String::from("0"),
                    b => panic!("mangled boole value: {}", b)
                })
            }
            SexprKind::RealLiteral | SexprKind::IntegerLiteral | SexprKind::StringLiteral => {
                ValRepr::Simple(self.token.get_text(source).to_string())
            },
        })
    }
}


pub fn build_global_sexprs(source: &str, all_scopes: &Vec<Scope>, sexprs: &Vec<Sexpr>, function_manager: &FunctionManager) -> Result<String, Error> {
    let mut build_flags = BuildFlags::new();
    let mut prgm = String::new();
    for ref sexpr in sexprs {
        sexpr.build(&mut prgm, source, all_scopes, function_manager, &mut build_flags)?;
    }
    build_func_defs(function_manager, &mut prgm, source, all_scopes, &mut build_flags)?;

    let mut header = String::new();
    build_header(&mut header, &build_flags);
    header.push_str(&prgm);
    Ok(header)
}

///get the initializations that might be called for
fn build_header(header: &mut String, build_flags: &BuildFlags) {
    if build_flags.initialize_master_string {
        header.push_str("\"\"→Str0\n");
    }
    if build_flags.initialize_stack_frames {
        header.push_str("{}→⌊RES\n{}→⌊ARGS\n")
    }
}

///appends the function definitions (with goto + labels) to the code of the program
fn build_func_defs(function_manager: &FunctionManager, prgm: &mut String, source: &str, all_scopes: &Vec<Scope>, build_flags: &mut BuildFlags) -> Result<(), Error> {
    if function_manager.count == 0 {
        return Ok(());
    }
    build_flags.initialize_stack_frames = true;
    prgm.push_str("Return\n");
    for id in 0..function_manager.count {
        build_func(function_manager, id, prgm, source, all_scopes, build_flags)?;
    }
    Ok(())
}
///append the goto + function body code to the code of the program
fn build_func(function_manager: &FunctionManager, id: usize, prgm: &mut String, source: &str, all_scopes: &Vec<Scope>, build_flags: &mut BuildFlags) -> Result<(), Error> {
    prgm.push_str(&format!("Lbl {}\n", function_manager.func_labels[id]));
    let body_repr: ValRepr = function_manager.func_body[id].build(prgm, source, all_scopes, function_manager, build_flags)?;
    if !function_manager.func_out_type[id].is_void() {
        // push the handle to the result stack
        prgm.push_str(&format!("{}→⌊RES(dim(⌊RES)+1)\n", body_repr.read()));
    }
    // jump to appropriate place
    //TODO optimizations on function call backs
    for (call_id, call_label) in function_manager.call_labels.iter().enumerate() {
        prgm.push_str(&format!("If ⌊ARGS(dim(⌊ARGS))={}\nGoto {}\n", call_id, call_label));
    }
    Ok(())
}