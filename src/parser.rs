use util::Error;
use tokenizer::Token;
use manager::Manager;
use type_checker::FutureType;
use sexpr::{Sexpr, SexprId, SexprKind};
use std::vec::Drain;
use std::collections::VecDeque;
use std::iter::Peekable;


/// given a token's text, make its corresponding atom kind
fn make_atom(text: &str) -> Option<SexprKind> {
    Some(if &text[0..1] == "\"" {
        SexprKind::StringLiteral
    }
        else if text == "true" || text == "false" {
            SexprKind::BooleLiteral
        }
            else if text.parse::<i32>().is_ok() {
                SexprKind::IntegerLiteral
            }
                else if text.parse::<f32>().is_ok() {
                    SexprKind::RealLiteral
                }
                    else if text.chars().nth(0).map_or(false, |ch| !ch.is_numeric()) && text.chars().all(|ch| ch == '-' || ch == '_' || ch == '<' || ch  == '>' || ch.is_alphanumeric()) {
                        // first character must exist and be non numeric, rest of the characters can alphanumeric, dash, or underscore
                        SexprKind::Identifier
                    } else {
                        return None;
                    })
}

fn parse_name_type_pairs_until(source: &str, head: Token, tokens: &mut Peekable<Drain<Token>>, closing_token_text: &str) -> Result<(Vec<String>, Vec<FutureType>), Error> {
    let mut arguments = vec![];
    let mut signature = vec![];
    loop {
        let first_token: Token = tokens.next().ok_or(Error::new("expected argument name, found end of file".to_owned(), head))?;

        if first_token.get_text(source) == closing_token_text {
            break;
        }

        let second_token: Token = tokens.next().ok_or(Error::new("expected type separator `:`, found end of file".to_owned(), head))?;
        if second_token.get_text(source) != ":" {
            return Err(Error::new(format!("expected type separator `:`, found `{}`", second_token.get_text(source)), second_token))
        }

        let third_token = tokens.next().ok_or(Error::new("expected argument type, found end of file".to_owned(), first_token))?;

        arguments.push(first_token.get_text(source).to_string());
        signature.push(FutureType::new(third_token));

    }
    Ok((arguments, signature))
}

impl Manager {
    fn push_new_sexpr(&mut self, kind: SexprKind, token: Token) -> SexprId {
        use std::cell::RefCell;
        self.all_sexprs.push(RefCell::new(Sexpr::new(Box::new(kind), token)));
        (self.all_sexprs.len() -1).into()
    }

    /// parses a s-expr from the tokens, which expects at least one of
    fn parse(&mut self, tokens: &mut Peekable<Drain<Token>>) -> Result<SexprId, Error> {
        let token = tokens.next().unwrap();
        let text = &token.get_text(&self.source).to_owned();
        let (kind, token) =
            if text == "(" {
                // we have a compound type
                if tokens.peek().is_none() {
                    return Err(Error::new(format!("unclosed s-expression"), token))
                }
                let head = tokens.next().unwrap();
                let head_str = head.get_text(&self.source).to_owned();
                match &head_str[..] {
                    "(" | ")" => {
                        return Err(Error::new(format!("illegal head of s-expression"), head));
                    }
                    "func" => {
                        // handle the function definition parsing elsewhere
                        return self.parse_function_definition(head, tokens);
                    }
                    "struct" => {
                        // handle the struct definition parsing elsewhere
                        return self.parse_struct_definition(head,tokens);
                    }
                    _ => {}
                }
                let mut tail = VecDeque::new();
                // keep eating things up while the next token exists and is not the closing )
                let compound_sexpr_token =
                    loop {
                        if let Some(peeked_token) = tokens.peek() {
                            if peeked_token.get_text(&self.source) == ")" {
                                // what a gentleman. the s-expr has been closed
                                break (self.make_compound(head, tail)?, head);
                            } else {
                                // continue parsing
                            }
                        } else {
                            // the s-expr has been left un-closed
                            return Err(Error::new(format!("unclosed s-expression"), token));
                        }
                        tail.push_back(self.parse(tokens)?);
                    };
                // eat the closing (
                tokens.next();
                compound_sexpr_token
            } else if text == ")" {
                return Err(Error::new(format!("un-paired closing parenthesis"), token));
            } else {
                // we have an atom
                let atom = make_atom(text).ok_or(Error::new(format!("not a recognized keyword, literal, or identifier"), token))?;
                (atom, token)
            };
        Ok(self.push_new_sexpr(kind, token))
    }

    /// given an s-expression head and tail, construct the corresponding compound
    fn make_compound(&mut self, head: Token, mut tail: VecDeque<SexprId>) -> Result<SexprKind, Error> {
        let name = head.get_text(&self.source);
        Ok(match name {
            // we must simply settle the particulars
            "declare"   => {
                if tail.len() < 3 { //TODO type inference
                    return Err(Error::new(format!("declare expected at least 3 arguments, not {}", tail.len()), head));
                }
                let variable_pattern = self.get_ident(tail.pop_front().unwrap())?;
                let expr = tail.pop_front().unwrap();
                let body = self.push_new_sexpr(
                    SexprKind::Block{statements: tail}, head
                );
                SexprKind::Declare { variable_pattern, expr, body }
            },
            "assign" => {
                if tail.len() != 2 {
                    return Err(Error::new(format!("assign expected exactly 2 arguments"), head));
                }
                let variable_pattern = self.get_ident(tail.pop_front().unwrap())?;
                let expr = tail.pop_front().unwrap();
                SexprKind::Assign { variable_pattern, expr }
            }
            "if" => {
                if tail.len() != 3 {
                    return Err(Error::new(format!("if expected exactly 3 arguments"), head));
                }
                SexprKind::IfSwitch {
                    predicate: tail.pop_front().unwrap(),
                    if_branch: tail.pop_front().unwrap(),
                    else_branch: tail.pop_front().unwrap(),
                }
            }
            "while" => {
                if tail.len() < 1 {
                    return Err(Error::new(format!("while expected at least 1 argument"), head));
                }
                SexprKind::WhileLoop {
                    predicate: tail.pop_front().unwrap(),
                    body: self.push_new_sexpr(
                        SexprKind::Block { statements: tail }, head
                    )
                }
            }
            "get-field" => {
                if tail.len() != 2 {
                    return Err(Error::new(format!("get expected exactly 2 arguments"), head));
                }
                let expr = tail.pop_front().unwrap();
                let field = self.get_ident(tail.pop_front().unwrap())?;
                SexprKind::StructGet { id: None, expr, field }
            }
            "set-field" => {
                if tail.len() != 3 {
                    return Err(Error::new(format!("set expected exactly 3 arguments"), head));
                }
                let expr = tail.pop_front().unwrap();
                let field = self.get_ident(tail.pop_front().unwrap())?;
                let value = tail.pop_front().unwrap();
                SexprKind::StructSet { id: None, expr, field, value }
            }
            "format" => {
                SexprKind::Format { exprs: tail }
            }
            "block" => {
                SexprKind::Block {
                    statements: tail
                }
            }
            _ => {
                SexprKind::Other{
                    opt_exprs: Some(tail)
                }
            },
        })
    }

    fn parse_struct_definition(&mut self, head: Token, tokens: &mut Peekable<Drain<Token>>) -> Result<SexprId, Error> {
        let name_token = tokens.next().ok_or(Error::new("unexpected end of file while scanning struct definition: missing a name".to_string(), head))?;
        let name = String::from(name_token.get_text(&self.source));
        match make_atom(&name) {
            Some(SexprKind::Identifier) => {},
            _ => {
                return Err(Error::new("invalid struct name: must be a proper identifier".to_string(), name_token));
            }
        }
        let (arguments, signature) = parse_name_type_pairs_until(&self.source, head, tokens, ")")?;
        Ok(self.push_new_sexpr(
            SexprKind::StructDef{id: self.udt_manager.declare_type(head, name, arguments, signature)?},
            head
        ))
    }

    fn parse_function_definition(&mut self, head: Token, tokens: &mut Peekable<Drain<Token>>) -> Result<SexprId, Error> {
        // where we are currently at:
        // how to deal with function definitions
        // 1) if it wants a special form in the way it's defined could be something like (func name arg:type arg:type arg:type.. -> type body...
        // 2) the more conservative approach is (func name (arg type arg type...) type body...)
        //  1 requires functions to be treated differently: this might be quite hard to do with how close parens are handled
        //  1 looks much nicer and is clearer
        //  2 requires that we let the head be anything at all and then kill if it's not a function
        let name_token = tokens.next().ok_or(Error::new("unexpected end of file while scanning function definition: missing a name".to_string(), head))?;
        let name = name_token.get_text(&self.source).to_string();
        if let Some(SexprKind::Identifier) = make_atom(&name) { }
            else {
                return Err(Error::new("invalid function name: must be a proper identifier".to_string(), name_token));
            }

        let (arguments, signature) = parse_name_type_pairs_until(&self.source, head, tokens, "->")?;
        let out_token = tokens.next().ok_or(Error::new(format!("unexpected end of file while scanning function definition (expected out type after -> )"), head))?;
        let out_type = FutureType::new(out_token);
        let mut statements = VecDeque::new();
        loop {
            {
                let token = tokens
                    .peek()
                    .ok_or(Error::new(
                        "unexpected end of file while scanning function definition expected closing parenthesis".to_owned(),
                        head))?;
                if token.get_text(&self.source) == ")" {
                    break;
                }
            }
            statements.push_back(self.parse(tokens)?);
        }
        tokens.next(); // eat up the closing )
        let body = self.push_new_sexpr(
            SexprKind::Block{ statements }, head
        );
        Ok(self.push_new_sexpr(
            SexprKind::FuncDef { func_id: self.func_manager.declare_func(name, arguments, signature, out_type, body)? },
            head
        ))
    }
}

pub fn generate_global_sexprs(m: &mut Manager, mut tokens: Vec<Token>) -> Result<(), Error> {
    //TODO get this to return multiple errors
    //    println!("running method generate_global_sexprs length: {}", tokens.len());
    let mut top_level_sexprs = vec![];
    let mut iterator = tokens.drain(..).peekable();
    while iterator.peek().is_some() {
        top_level_sexprs.push(m.parse(&mut iterator)? );
    }
    m.top_level_sexprs = top_level_sexprs;
    Ok(())
}