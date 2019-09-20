/*
use sexpr::{Sexpr, SexprKind, SexprId};
use scope::Scope;
use util::vec_fmt;

use sexpr::SexprId;
use util::Error;
use type_checker::Type;
use manager::Manager;

pub struct BuildFlags {
    pub initialize_master_string: bool,
    pub initialize_stack_frames: bool,
    pub initialize_obj_mem: bool,
}

impl BuildFlags {
    fn new() -> BuildFlags {
        BuildFlags {
            initialize_master_string: false,
            initialize_stack_frames: false,
            initialize_obj_mem: false,
        }
    }
}

impl SexprId {
    ///return a pair of `String`s: (handle, code)
    /// handle - what other pieces of code should use to refer to the result of this s-expr
    /// code - what other pieces of code need to run before refering to the result of this s-expr
    fn build(self, m: &Manager, prgm: &mut String, build_flags: &mut BuildFlags) -> Result<ValRepr, Error> {
        unimplemented!()
        // generate the (handle, code) tuple
        //WARN the code must end in a newline
        /*Ok(match *self.kind {
            SexprKind::Declare { ref variable_pattern, expr, body } => {
                let variable = Scope::lookup_val_repr(all_scopes,body.scope.unwrap(), variable_name);
                let expr_handle = expr.build(prgm, source, all_scopes, m, build_flags)?;
                variable.write(prgm, &expr_handle);
                body.build(prgm, source, all_scopes, m, build_flags)?
            },
            SexprKind::Assign { ref variable_pattern, ref expr } => {
                let variable = Scope::lookup_val_repr(all_scopes,self.scope.unwrap(), variable_name);
                let expr_handle = expr.build(prgm, source, all_scopes, m, build_flags)?;
                variable.write(prgm, &expr_handle);
                variable
            },
            SexprKind::IfSwitch { ref predicate, ref if_branch, ref else_branch } => {
                let variable = self.associated_reprs[0].clone();

                let predicate_repr = predicate.build(prgm, source, all_scopes, m, build_flags)?;
                prgm.push_str(&format!("If {}\nThen\n", predicate_repr.read()));

                let if_repr = if_branch.build(prgm, source, all_scopes, m, build_flags)?;
                variable.write(prgm, &if_repr);
                prgm.push_str("Else\n");

                let else_repr = else_branch.build(prgm, source, all_scopes, m, build_flags)?;
                variable.write(prgm, &else_repr);
                prgm.push_str("End\n");

                variable
            },
            SexprKind::WhileLoop { ref predicate, ref body } => {
                let predicate_repr = predicate.build(prgm, source, all_scopes, m, build_flags)?;
                prgm.push_str(&format!("While {}\n", predicate_repr.read()));
                let body_repr = body.build(prgm, source, all_scopes, m, build_flags)?;
                prgm.push_str("End\n");
                body_repr
            },
            SexprKind::List { ref elements } => {
                let var = self.associated_reprs[0].clone();
                prgm.push_str(&format!("{len}→dim({var})\n", len = elements.len(), var = var.read()));
                for (index, elem) in elements.iter().enumerate() {
                    let mut elem_repr = elem.build(prgm, source, all_scopes, m, build_flags)?;
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
                let list_repr = list.build(prgm, source, all_scopes, m, build_flags)?;
                let index_repr = index.build(prgm, source, all_scopes, m, build_flags)?;
                let handle = format!("{list}({i})", list = list_repr.read(), i = index_repr.read());
                ValRepr::interpret_num(handle, self.return_type.as_ref().unwrap())
            }
            SexprKind::ListSet { ref list, ref index, ref elem } => {
                let list_repr = list.build(prgm, source, all_scopes, m, build_flags)?;
                let index_repr = index.build(prgm, source, all_scopes, m, build_flags)?;
                let elem_handle = elem
                    .build(prgm, source, all_scopes, m, build_flags)?
                    .transmute_num(elem.return_type.as_ref().unwrap(), prgm, build_flags)
                    .ok_or(Error::new(format!("Can not store value of type {} in a list (must have a valid numeric representation)", elem.return_type.as_ref().unwrap()), elem.token))?;

                prgm.push_str(&format!("{elem_handle}→{list_repr}({index_repr})\n",
                                       elem_handle = elem_handle,
                                       list_repr = list_repr.read(),
                                       index_repr = index_repr.read(),
                ));
                ValRepr::new_void()
            }
            SexprKind::Block { ref statements } => {
                let mut repr = ValRepr::new_void();
                for ref statement in statements {
                    // TODO mark unused statements as such <- why is this here? we need to do it in type-checking
                    repr = statement.build(prgm, source, all_scopes, m, build_flags)?;
                }
                repr
            },
            SexprKind::FuncDef { id: _ } | SexprKind::StructDef { id: _ } => {
                // definitions have no in-place code
                ValRepr::new_void()
            },
            SexprKind::FuncCall { ref func_id, ref call_id, ref exprs } => {
                // we want to push onto ⌊ARGS: everything in the scope of the function plus our call_id
                let func_scope_index = m.get_func_body_ref(*func_id).scope.unwrap();
                let var_total = Scope::count_total_vars(all_scopes, func_scope_index);
                // functions look like:
                prgm.push_str(&format!("{}→dim(⌊AUX\n", var_total + 1));
                // our call back identifier is on top of the stack
                prgm.push_str(&format!("{}→⌊AUX({})\n", call_id, var_total + 1));
                // initialize all the arguments
                for (ref index, ref expr) in exprs.iter().enumerate() {
                    let mut handle = expr
                        .build(prgm, source, all_scopes, m, build_flags)?
                        .transmute_num(expr.return_type.as_ref().unwrap(), prgm, build_flags)
                        .ok_or(Error::new("Can not pass value of this type to function".to_owned(), expr.token))?; //TODO do this error handling in function definition
                    prgm.push_str(&format!("{}→⌊AUX({})\n", handle, var_total - index))
                }
                prgm.push_str(&format!("augment(⌊ARGS,⌊AUX→⌊ARGS\nGoto {func_label}\nLbl {call_label}\n",
                                       func_label = m.func_labels[*func_id],
                                       call_label = m.call_labels[*call_id]));
                prgm.push_str(&format!("dim(⌊ARGS)-{size}→dim(⌊ARGS\n",
                                       size = var_total + 1, // all our variables (locals + args) + the call bac
                ));
                //TODO remove this when we implement smart repr assigning
                // assign the top of the result stack to our label
                self.associated_reprs[0]
                    .write(prgm, &ValRepr::interpret_num(
                        "⌊RES(dim(⌊RES))".to_owned(),
                        self.return_type.as_ref().unwrap()));
                self.associated_reprs[0].clone()
            }
            SexprKind::StructInit { id:_, ref exprs } => {
                // build a struct initialization

                let expr_handles = exprs
                    .iter()
                    .map(|expr: &Sexpr|
                        expr
                            .build(prgm, source, all_scopes, m, build_flags)
                            .and_then(|val_repr: ValRepr| val_repr
                                .transmute_num(expr.return_type.as_ref().unwrap(), prgm, build_flags)
                                .ok_or(Error::new(format!("Can not store value of type {} in a struct (must have a valid numeric representation)", expr.return_type.as_ref().unwrap()), expr.token))
                            )
                    )
                    .collect::<Result<Vec<String>, Error>>()?;

                build_flags.initialize_obj_mem = true;
                prgm.push_str(&format!("dim(⌊OBJ)+1→{}\n", self.associated_reprs[0].read())); // set our result to the beginning of our segment
                for ref handle in expr_handles {
                    prgm.push_str(&format!("{}→⌊OBJ(1+dim(⌊OBJ\n", handle));
                }
                self.associated_reprs[0].clone()
            }
            SexprKind::StructGet { ref id, ref expr, ref field } => {
                let repr: ValRepr = expr.build(prgm, source, all_scopes, m, build_flags)?;
                let offset = m.get_field_offset(id.expect("struct id should have been logged in typechecking"), field);
                ValRepr::Simple(
                    format!("⌊OBJ({index}+{offset})",
                            index = repr.read(),
                            offset = offset
                    )
                )
            },
            SexprKind::StructSet { ref id, ref expr, ref field, ref value } => {
                let struct_repr: ValRepr = expr.build(prgm, source, all_scopes, m, build_flags)?;
                let value_handle = value
                    .build(prgm, source, all_scopes, m, build_flags)?
                    .transmute_num(value.return_type.as_ref().unwrap(), prgm, build_flags)
                    .expect("we made a struct with non num transmutable types");

                let offset = m.udt_manager.get_field_offset(id.expect("struct id should have been logged in typechecking"), field);
                prgm.push_str(
                    &format!("{value}→⌊OBJ({index}+{offset})\n",
                        index = struct_repr.read(),
                        offset = offset,
                        value = value_handle,

                ));
                ValRepr::new_void()
            },
            SexprKind::Format { ref exprs } => {
                build_flags.initialize_master_string = true;
                let mut val_reprs = vec![];
                for ref expr in exprs {
                    val_reprs.push(expr.build(prgm, source, all_scopes, m, build_flags)?);
                }
                let my_repr = &self.associated_reprs[0];
                prgm.push_str(&format!("length(Str0)+1→{}\n", my_repr.get_tag()));
                for index in 0..exprs.len() {
                    build_stringification(
                        &val_reprs[index],
                        exprs[index].return_type.as_ref().unwrap(),
                        Some(&self.associated_reprs[1]),
                        prgm,
                        m)?;
                }

                prgm.push_str(&format!("{s}+(1+length(Str0)-{s})/9→{s}\n", s = my_repr.get_tag()));
                my_repr.clone()
            }
            SexprKind::BuiltIn { ref id, ref exprs } => {
                let mut val_reprs = vec![];
                for ref expr in exprs {
                    val_reprs.push(expr.build(prgm, source, all_scopes, m, build_flags)?);
                }
                let read_only_handles: Vec<String> = val_reprs.iter().map(|vr| vr.read()).collect();
                let additional_code = vec_fmt(&m.builtin_manager.builtin_code[*id], &read_only_handles);
                prgm.push_str(&additional_code);
                let my_handle = vec_fmt(&m.builtin_manager.builtin_handle[*id], &read_only_handles);
                ValRepr::Simple(my_handle)
            }
            SexprKind::Other { opt_exprs: _ } => panic!("we should not be building a SexprKind::Other. self = {:#?}", self),
            SexprKind::Identifier => {
                Scope::lookup_val_repr(all_scopes,self.scope.unwrap(), self.token.get_text(source))
            },
            SexprKind::BooleLiteral => {
                ValRepr::Simple(match self.token.get_text(source) {
                    "true"  => String::from("1"),
                    "false" => String::from("0"),
                    b => panic!("mangled boole value: {}", b)
                })
            }
            SexprKind::RealLiteral | SexprKind::IntegerLiteral => {
                // replace the minus sign with negative sign ()
                ValRepr::Simple(self.token.get_text(source).replace("-", "­"))
            }
            SexprKind::StringLiteral => {
                ValRepr::Simple(self.token.get_text(source).to_string())
            },
        })*/
    }
}


pub fn build_global_sexprs(m: &Manager) -> Result<String, Error> {
    let mut build_flags = BuildFlags::new();
    let mut prgm = String::new();
    let mut repr: Option<ValRepr> = None;
    for sexpr_id in m.top_level_sexprs {
        repr = Some(sexpr_id.build(m, &mut prgm, &mut build_flags)?);
    }
    repr.map(|r: ValRepr| prgm.push_str(&r.read()));

    build_func_defs(m, &mut prgm, &mut build_flags)?;

    let mut header = String::new();
    build_header(&mut header, &build_flags);
    header.push_str(&prgm);
    Ok(header)
}

///get the initializations that might be called for
fn build_header(header: &mut String, build_flags: &BuildFlags) {
    if build_flags.initialize_master_string {
        header.push_str("\" \"→Str0\n");
    }
    if build_flags.initialize_stack_frames {
        header.push_str("{0}→⌊RES\n{0}→⌊ARGS\n")
    }
    if build_flags.initialize_obj_mem {
        header.push_str("{0}→⌊OBJ\n");
    }
}

///appends the function definitions (with goto + labels) to the code of the program
fn build_func_defs(m: &Manager, prgm: &mut String, build_flags: &mut BuildFlags) -> Result<(), Error> {
    if m.func_manager.count == 0 {
        return Ok(());
    }
    build_flags.initialize_stack_frames = true;
    prgm.push_str("\nReturn\n");
    for id in 0..m.func_manager.count {
        build_func(m, id, prgm, build_flags)?;
    }
    Ok(())
}

///append the goto + function body code to the code of the program
fn build_func(m: &Manager, id: usize, prgm: &mut String, build_flags: &mut BuildFlags) -> Result<(), Error> {
    prgm.push_str(&format!("Lbl {}\n", m.func_manager.func_labels[id]));
    let body_repr: ValRepr = m.func_manager.func_body(id).build(m, prgm, build_flags)?;
    if !m.func_manager.out_type[id].unwrap().is_void() {
        // push the handle to the result stack
        let handle = body_repr
            .transmute_num(m.func_manager.out_type[id].unwrap(), prgm, build_flags)
            .ok_or(Error::new("Can not pass value of this type to function".to_owned(), self.sexpr(m.func_manager.func_body(id)).token))?; //TODO do this error handling in function definition
        prgm.push_str(&format!("{}→⌊RES(dim(⌊RES)+1)\n", handle));
    }
    // jump to appropriate place
    //TODO optimizations on function call backs
    for (call_id, call_label) in m.func_manager.call_labels.iter().enumerate() {
        prgm.push_str(&format!("If ⌊ARGS(dim(⌊ARGS))={}\nGoto {}\n", call_id, call_label));
    }
    Ok(())
}

///append the code needed to convert a given value to a String
/// the program will add the stringified version onto the end of Str0
fn build_stringification(target: &ValRepr, arg_type: &Type, maybe_repr: Option<&ValRepr>, prgm: &mut String, manager: &Manager) -> Result<(), Error> {
    // lazily read the maybe_repr, returning an error if we don't have a util_repr to use
    let read_util = || {
        maybe_repr
            .ok_or(Error::new_zero("stringication only supports 2 layers".to_string()))
            .map(ValRepr::read)
    };
    match arg_type {
        Type::String => {
            prgm.push_str(&format!("Str0+{}→Str0\n", target.read()));
        },
        Type::Void => {
            prgm.push_str("Str0+\"void\"→Str0\n");
        },
        Type::Boole => {
            prgm.push_str(&format!(
                "\
If {target}
Str0+\"true\"→Str0
If not({target}
Str0+\"false\"→Str0
",
                target = target.read()));
        },
        Type::Int => {
            prgm.push_str(&format!(
                "\
0→{util}
If {target}>0
{target}/^(1+iPart(log({target}→{util}
If {target}<0
­{target}/^(1+iPart(log(­{target}→{util}
If {target}<0
Str0+\"­\"→Str0
If {target}=0
Str0+\"0\"→Str0
While {util}>0
Str0+sub(\"0123456789\",iPart(10fPart({util}))+1,1→Str0
fPart(10fPart({util}→{util}
End
",
            target = target.read(), util = read_util()?));
        },
        Type::Real => {
            // do the int part
            prgm.push_str(&format!(
                "\
0→{util}
If {target}>0
{target}/^(1+iPart(log(iPart({target}→{util}
If {target}<0
­{target}/^(1+iPart(log(­iPart({target}→{util}
If {target}<0
Str0+\"­\"→Str0
If {target}=0
Str0+\"0\"→Str0
While {util}>0
Str0+sub(\"0123456789\",iPart(10fPart({util}))+1,1→Str0
fPart(10fPart({util}→{util}
End
",
                target = target.read(), util = read_util()?));
            // put the decimal
            prgm.push_str("Str0+\".\"→Str0\n");
            // do the fractional part
            prgm.push_str(&format!(
                "\
0→{util}
If {target}>0
fPart({target})→{util}
If {target}<0
­fPart({target})→{util}
If {target}=0
Str0+\"0\"→Str0
While {util}>0
Str0+sub(\"0123456789\",iPart(10fPart({util}))+1,1→Str0
fPart(10fPart({util}→{util}
End
",
                target = target.read(), util = read_util()?));
        },
        // compound types are harder
        Type::List(ref inner_type) => {
            // initial set up + first half of loop
            prgm.push_str(&format!(
                "\
Str0+\"{{\"→Str0
1→{util}
While {util}≤dim({target})
If {util}≠1
Str0+\" \"→Str0
",
                target = target.read(), util = read_util()?));
            // the code for each element
            build_stringification(
                &ValRepr::interpret_num(format!("{list}({index})", list = target.read(), index = read_util()?), inner_type),
                inner_type,
                None,
                prgm,
                manager
            )?;
            // final tear down + Second half of loop
            prgm.push_str(&format!(
                "\
1+{util}→{util}
End
Str0+\"}}\"→Str0
",
                util = read_util()?));
        },
        Type::CustomType(ref name, _) => {
            //TODO if this type has an to-string function, we should call it
            //for now, we will content our selves with a peek at the shallow pointer
            // we know its idx is a positive integer
            prgm.push_str(&format!("\
Str0+\"<Struct {typename}, idx: \"→Str0
0→{util}
{target}/^(1+iPart(log({target}→{util}
While {util}>0
Str0+sub(\"0123456789\",iPart(10fPart({util}))+1,1→Str0
fPart(10fPart({util}→{util}
End
Str0+\">\"→Str0
\
", typename = name, util = read_util()?, target = target.read()));
        }
    }
    Ok(())
}
*/