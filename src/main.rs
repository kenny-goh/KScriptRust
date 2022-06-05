extern crate core;

use std::{env, fs, mem};
use std::process::exit;
use std::time::{Instant};

use crate::chunk::{Chunk, Opcode};
use crate::compiler::Parser;
use crate::heap::Heap;
use crate::object::Object;
use crate::scanner::Scanner;
use crate::utils::read_line;
use crate::value::Value;
use crate::vm::{RunResult, VM};

mod value;
mod chunk;
mod object;
mod function;
mod token;
mod vm;
mod callframe;
mod scanner;
mod compiler;
mod heap;
mod utils;
mod debug;
mod nativefn;
mod closure;
mod class;
mod instance;
mod tests;

/// Main entry point to KScript VM
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        run_prompt();
    } else if args.len() == 2 {
        let filename = args.get(1).unwrap();
        run_file(filename);
    }
}

/// EVAL loop mode
fn run_prompt() {
    let mut vm = VM::new();
    vm.init();
    println!("KScript VM written in RUST :)");
    loop {
        println!("> ");
        let result = read_line();
        let line  = match result {
            Ok(line) => line,
            Err(error) => panic!("Unable to read input {}", error),
        };
        if line.trim() == "" {
            continue;
        }
        else if line.trim() == "exit" {
            println!("Good bye!\n");
            break;
        }
        vm = run(vm, &line);
        vm.reset_stack();
    }
}

/// Execute the VM by loading the KScript from file
fn run_file(filename: &String) {

    let source = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let mut vm = VM::new();
    vm.init();

    let mut scanner = Scanner::new(  &source);
    let tokens = scanner.scan_tokens();

    // transfer heap ownership to parser
    let mut heap_to_parser = Heap::new();
    mem::swap(&mut vm.heap, &mut heap_to_parser);

    let mut parser = Parser::new(heap_to_parser, tokens);
    parser.compile();

    // transfer heap ownership of back to vm
    mem::swap(&mut parser.heap, &mut vm.heap,);

    // Bail out on parser error
    if parser.had_error {  exit(50);}

    let start = Instant::now();
    let result = vm.execute();
    let duration = start.elapsed();

    match result {
        RunResult::RuntimeError => { exit(70)}
        RunResult::Ok => {
            println!("Time elapsed interpret is: {:?}", duration);
            exit(0);
        }
    }
}

///
fn run(mut vm: VM, source: &String) ->VM {

    let mut scanner = Scanner::new( source);

    let tokens = scanner.scan_tokens();

    // transfer heap ownership of heap in VM to heap1
    let mut heap1 = Heap::new();
    mem::swap(&mut vm.heap, &mut heap1);

    let mut parser = Parser::new(heap1, tokens);
    parser.compile();

    // transfer heap ownership of back to vm
    mem::swap(&mut parser.heap, &mut vm.heap,);

    if !parser.had_error {
        vm.execute();
    }

    return vm;
}




