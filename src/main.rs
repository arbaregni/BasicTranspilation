#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;

mod util;
mod lang_consts;
mod tokenizer;
mod sexprizer;
mod scoper;
mod type_checker;
mod variablizer;
mod functionizer;
mod builder;

use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::stdout;

fn read_arguments() -> (String, Option<String>, bool) {
    // actual version
    // env::args().nth(1).ok_or(Error::new(ErrorKind::NotFound, "missing command line argument"))?;
    let in_path = env::args().nth(1).unwrap_or("test.txt".to_owned());
    let mut maybe_out_path: Option<String> = None;
    let mut debug = false;
    for arg in env::args().into_iter().skip(2) {
        match arg.as_ref() {
            "-D" | "-d" if !debug => {
                debug = true;
            }
            _ => {
                maybe_out_path = Some(arg.clone());
            }
        }
    }
    (in_path, maybe_out_path, debug)
}

fn read_source(path: String) -> Result<String, std::io::Error> {
    let mut source = String::new();
    let mut file = match File::open(path) {
        Ok(f)   => f,
        Err(e)  => {
            return Err(e);
        }
    };
    match file.read_to_string(&mut source) {
        Ok(_)   => {},
        Err(e)  => {
            return Err(e);
        }
    }
    return Ok(source);
}

fn write_or_display(maybe_path: Option<String>, text: String) -> Result<(), std::io::Error> {
    let mut out_writer: Box<Write> = match maybe_path {
        Some(ref x) => Box::new(File::create(&Path::new(x))?),
        None => Box::new(stdout()),
    };
    out_writer.write(text.as_bytes())?;
    Ok(())
}

fn main() {
    let (in_path, maybe_out_path, debug) = read_arguments();

    println!("compiling {in_path} to {out}\nwith flags: debug = {debug}",
        in_path = in_path,
        out = maybe_out_path.as_ref().map(|x| &**x).unwrap_or("stdout"),
        debug = debug
    );

    let source = match read_source(in_path) {
        Ok(source) => source,
        Err(error) => {
            return println!("{}", error);
        }
    };

    let tokens = tokenizer::tokenize(&source);
    let mut function_manager = functionizer::FunctionManager::new();

    let mut global_sexprs = match sexprizer::generate_global_sexprs(&source, tokens, &mut function_manager) {
        Ok(global_sexprs) => global_sexprs,
        Err(error) => {
            return println!("{}", error.get_readout(&source));
        }
    };

    let mut all_scopes = match scoper::create_all_scopes(&source, &mut global_sexprs, &mut function_manager) {
        Ok(all_scopes) => all_scopes,
        Err(error) => {
            return println!("{}", error.get_readout(&source));
        },
    };

    match type_checker::type_check_all(&source, &mut all_scopes, &mut global_sexprs, &mut function_manager) {
        Ok(_) => {},
        Err(error) => {
            return println!("{}", error.get_readout(&source));
        },
    }

    match variablizer::create_all_variables(&source, &mut all_scopes, &mut global_sexprs, &mut function_manager) {
        Ok(()) => {},
        Err(error) => {
            return println!("{}", error.get_readout(&source));
        }
    }

    if debug {
        println!("{:#?}", global_sexprs);
        println!("{:#?}", all_scopes);
        println!("{:#?}", function_manager);
    }

    let text = match builder::build_global_sexprs(&source, &all_scopes, &global_sexprs, &mut function_manager) {
            Ok(text) => text,
            Err(error) => {
                return println!("{}", error.get_readout(&source));
            }
        };

    match write_or_display(maybe_out_path, text) {
        Ok(()) => {},
        Err(error) => {
            return println!("{}", error);
        }
    }

    println!("compilation was successful");
}
