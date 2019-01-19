use util::Error;
use tokenizer::Token;
use type_checker::Type;
use functionizer::FunctionManager;
use variablizer::ValRepr;
use std::vec::Drain;
use std::collections::VecDeque;
use std::iter::Peekable;

#[derive(Debug)]
pub enum OtherKind {
    Undecided,
    BuiltIn{id: usize},
    FuncCall{func_id: usize, call_id: usize},
}

#[derive(Debug)]
pub enum SexprKind {
    StringLiteral,
    IntegerLiteral,
    RealLiteral,
    BooleLiteral,
    Identifier,
    Declare{declare_type: Type, variable_name: String, expr: Sexpr, body: Sexpr},
    Assign{variable_name: String, expr: Sexpr},
    IfSwitch{predicate: Sexpr, if_branch: Sexpr, else_branch: Sexpr},
    WhileLoop{predicate: Sexpr, body: Sexpr},
    List{elements: VecDeque<Sexpr>},
    ListGet{list: Sexpr, index: Sexpr},
    ListSet{list: Sexpr, index: Sexpr, elem: Sexpr},
    Block{statements: VecDeque<Sexpr>},
    FunctionDefinition{ id: usize },
    Other{name: String, kind: OtherKind, exprs: VecDeque<Sexpr>},
}

#[derive(Debug)]
pub struct Sexpr {
    pub kind: Box<SexprKind>,
    pub token: Token,
    pub scope: Option<usize>, // filled in during scoping
    pub return_type: Option<Type>, // filled in during type checking
    pub associated_reprs: Vec<ValRepr>, // might be used by things which need unbound variables
}
impl Sexpr {
    pub fn new(kind: Box<SexprKind>, token: Token) -> Sexpr {
        Sexpr {kind, token, scope: None, return_type: None, associated_reprs: vec![] }
    }
    pub fn dummy() -> Sexpr {
        Sexpr { kind: Box::new(SexprKind::BooleLiteral), token: Token::new(0, 0), scope: None, return_type: None, associated_reprs: vec![] }
    }
    ///returns a copy of its identifier if it is a valid one, otherwise returns an error
    pub fn identifier_or_else(&self, source: &str) -> Result<String, Error> {
        match *self.kind {
            SexprKind::Identifier => Ok(String::from(self.token.get_text(source))),
            _ => Err(Error::new(format!("not a valid identifier"), self.token))
        }
    }
}


pub fn generate_global_sexprs(source: &str, mut tokens: Vec<Token>, function_manager: &mut FunctionManager) -> Result<Vec<Sexpr>, Error> {
    //TODO get this to return multiple errors
    //    println!("running method generate_global_sexprs length: {}", tokens.len());
    let mut global_sexprs = vec![];
    let mut iterator = tokens.drain(..).peekable();
    while iterator.peek().is_some() {
        global_sexprs.push(parse(source, &mut iterator, function_manager)?);
    }
    Ok(global_sexprs)
}

/// parses a s-expr from the tokens, which expects at least one of
pub fn parse(source: &str, tokens: &mut Peekable<Drain<Token>>, function_manager: &mut FunctionManager) -> Result<Sexpr, Error> {
    let token = tokens.next().unwrap();
    let text = token.get_text(source);
    let (kind, token) =
        if text == "(" {
            // we have a compound type
            if tokens.peek().is_none() {
                return Err(Error::new(format!("unclosed s-expression"), token))
            }
            let head = tokens.next().unwrap();
            if head.get_text(source) == "(" || head.get_text(source) == ")" {
                return Err(Error::new(format!("illegal head of s-expression"), head));
            }
            if head.get_text(source) == "func" {
                // handle the function definition parsing elsewhere
                return parse_function_definition(source, head, tokens, function_manager);
            }
            let mut tail = VecDeque::new();
            // keep eating things up while the next token exists and is not the closing )
            let compound_sexpr_token =
                loop {
                    if let Some(peeked_token) = tokens.peek() {
                        if peeked_token.get_text(source) == ")" {
                            // what a gentleman. the s-expr has been closed
                            break (make_compound(source, head, tail)?, head);
                        } else {
                            // continue parsing
                        }
                    } else {
                        // the s-expr has been left un-closed
                        return Err(Error::new(format!("unclosed s-expression"), token));
                    }
                    tail.push_back(parse(source, tokens, function_manager)?);
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
    Ok(Sexpr::new(Box::new(kind), token))
}

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

/// given an s-expression head and tail, construct the corresponding compound
fn make_compound(source: &str, head: Token, mut tail: VecDeque<Sexpr>) -> Result<SexprKind, Error> {
    let name = head.get_text(source);
    Ok(match name {
        // we must simply settle the particulars
        "declare"   => {
            if tail.len() < 3 { //TODO type inference
                return Err(Error::new(format!("declare expected at least 3 arguments, not {}", tail.len()), head));
            }
            SexprKind::Declare {
                declare_type: {
                    let token = tail.pop_front().unwrap().token;
                    Type::from_text(source, token)?
                },
                variable_name: tail.pop_front().unwrap().identifier_or_else(source)?,
                expr: tail.pop_front().unwrap(),
                body: Sexpr::new(Box::new(SexprKind::Block { statements: tail }), head),
            }
        },
        "assign" => { //TODO is tail length funny? <-- what does this mean??
            if tail.len() != 2 {
                return Err(Error::new(format!("assign expected exactly 2 arguments"), head));
            }
            SexprKind::Assign {
                variable_name: tail.pop_front().unwrap().identifier_or_else(source)?,
                expr: tail.pop_front().unwrap(),
            }
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
                body: Sexpr::new(Box::new(SexprKind::Block { statements: tail }), head) //WARN tokens are messed up
            }
        }
        "list" => {
            SexprKind::List {
                elements: tail,
            }
        }
        "get" => {
            if tail.len() != 2 {
                return Err(Error::new(format!("get expected exactly 2 arguments"), head));
            }
            let list = tail.pop_front().unwrap();
            let index = tail.pop_front().unwrap();
            SexprKind::ListGet { list, index }
        }
        "set" => {
            if tail.len() != 3 {
                return Err(Error::new(format!("set expected exactly 3 arguments"), head));
            }
            let list = tail.pop_front().unwrap();
            let index = tail.pop_front().unwrap();
            let elem = tail.pop_front().unwrap();
            SexprKind::ListSet { list, index, elem }
        }
        "block" => {
            SexprKind::Block {
                statements: tail
            }
        }
        name => {
            SexprKind::Other{
                name: name.to_string(),
                kind: OtherKind::Undecided,
                exprs: tail
            }
        },
    })
}

fn parse_function_definition(source: &str, head: Token, tokens: &mut Peekable<Drain<Token>>, function_manager: &mut FunctionManager) -> Result<Sexpr, Error> {
    // where we are currently at:
    // how to deal with function definitions
    // 1) if it wants a special form in the way it's defined could be something like (func name arg:type arg:type arg:type.. -> type body...
    // 2) the more conservative approach is (func name (arg type arg type...) type body...)
    //  1 requires functions to be treated differently: this might be quite hard to do with how close parens are handled
    //  1 looks much nicer and is clearer
    //  2 requires that we let the head be anything at all and then kill if it's not a function
    let name_token = tokens.next().ok_or(Error::new("unexpected end of file while scanning function definition: missing a name".to_string(), head))?;
    let name = String::from(name_token.get_text(source));
    match make_atom(&name) {
        Some(SexprKind::Identifier) => {},
        _ => {
            return Err(Error::new("invalid function name: must be a proper identifier".to_string(), name_token));
        }
    }
    let mut arguments = vec![];
    let mut signature = vec![];
    let mut current_name = "";
    let mut have_completed_pair = false;
    loop {
        let maybe_token = tokens.next();
        if maybe_token.is_none() {
            return Err(Error::new(format!("unexpected end of file while scanning function definition: could not find -> to terminate argument list"), head));
        }
        if maybe_token.unwrap().get_text(source) == "->" {
            // the -> indicates the end of the arguments
            break;
        }
        if have_completed_pair {
            arguments.push(current_name.to_string());
            signature.push(Type::from_text(source, maybe_token.unwrap())?);
            have_completed_pair = false;
        } else {
            current_name = maybe_token.unwrap().get_text(source);
            have_completed_pair = true;
        }
    }
    let out_token = tokens.next().ok_or(Error::new(format!("unexpected end of file while scanning function definition (expected out type after -> )"), head))?;
    let out_type = Type::from_text(source, out_token)?;
    let mut statements = VecDeque::new();
    loop {
        {
            let maybe_token = tokens.peek();
            if maybe_token.is_none() {
                return Err(Error::new(format!("unexpected end of file while scanning function definition expected closing parenthesis"), head));
            }
            if maybe_token.unwrap().get_text(source) == ")" {
                break;
            }
        }
        statements.push_back( parse(source, tokens, function_manager)? );
    }
    tokens.next(); // eat up the closing )
    let body = Sexpr::new(
                Box::new(SexprKind::Block{ statements }),
               head
    );
    Ok(Sexpr::new(
        Box::new(SexprKind::FunctionDefinition { id: function_manager.declare_func(name, arguments, signature, out_type, body)? }),
              head
    ))
}