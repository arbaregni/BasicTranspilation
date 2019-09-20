#![allow(non_snake_case)]

extern crate take_mut;

mod scope;
mod util;
mod tokenizer;
mod parser;
mod type_checker;
mod dependencies;
mod sexpr;
mod scoping;
mod builder;
mod mir;
mod optimize;
mod manager;

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

fn read_file(path: String) -> Result<String, std::io::Error> {
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

    println!("compiling {in_path} to {out}\nwith flags: debug = {debug}\n------------------------------------",
        in_path = in_path,
        out = maybe_out_path.as_ref().map(|x| &**x).unwrap_or("stdout"),
        debug = debug
    );

    let source = read_file(in_path)
        .map_err(|error| return println!("{}", error))
        .unwrap();

    let mut m = manager::Manager::new(source);

    let tokens = tokenizer::tokenize(&m.source);

    parser::generate_global_sexprs(&mut m, tokens)
        .map_err(|error| return println!("Error lexing:\n{}", error.get_readout(&m))).unwrap();

    m.initialize_type_info()
        .map_err(|error| return println!("Error in initializing the user defined type info:\n{}", error.get_readout(&m))).unwrap();

    scoping::create_all_scopes(&mut m)
        .map_err(|error| return println!("Error in scoping:\n{}", error.get_readout(&m))).unwrap();

    type_checker::type_check_all(&mut m)
        .map_err(|error| return println!("Error in type checking:\n{}", error.get_readout(&m))).unwrap();

    mir::make_all_mir(&mut m)
        .map_err(|error| return println!("Error in generation medium internal representation:\n{}", error.get_readout(&m))).unwrap();

    let text = "".into(); // builder::build_global_sexprs(&mut m)
   //     .map_err(|error| return println!("Error in building:\n{}", error.get_readout(&m))).unwrap();

    if debug {
        println!("{:#?}", m)
    }

    write_or_display(maybe_out_path, text)
        .map_err(|error| return println!("{}", error)).unwrap();

    println!("------------------------------------\ncompilation was successful");
}
