#![allow(dead_code, unused)]

use std::ops::Index;
use crate::value::Value;

/**
 * Enum of op codes
 */
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Opcode {
    Constant = 0,
    Nil = 1,
    True = 2,
    False = 3,
    Pop = 4,
    GetLocal = 5,
    GetGlobal = 6,
    DefineGlobal = 7,
    SetLocal = 8,
    SetGlobal = 9,
    Equal = 10,
    GetUpvalue = 11,
    SetUpvalue = 12,
    Greater = 13,
    Less = 14,
    Add = 15,
    Subtract = 16,
    Multiply = 17,
    Divide = 18,
    Not = 19,
    Negate = 20,
    Print = 21,
    JumpIfFalse = 22,
    Jump = 23,
    Loop = 24,
    Call = 25,
    Closure = 26,
    CloseValue = 27,
    Return = 28,
}

impl Opcode {
    pub fn byte(&self) -> u8 {
        return *self as u8
    }
}

/// Represent a chunk of machine code
#[repr(C)]
#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>
}

impl Chunk {
    pub fn new() ->Self {
        Chunk {
            code: vec![],
            constants: vec![],
            lines: vec![]
        }
    }

    /// Append bytecode
    pub fn code(&mut self, byte: u8, line: usize) -> &mut Chunk {
        self.code.push(byte);
        self.lines.push(line);
        return self;
    }

    /// Add constant
    /// Return index number pointing to the constant
    pub fn add_constants(&mut self, val: Value) -> u8 {
        let existing_index = self.constants.iter().position(|&r| r == val );
        if existing_index.is_some()  {
            return existing_index.unwrap() as u8;
        }
        let index = self.constants.len() as u8; // fixme: might overflow!
        self.constants.push(val);
        return index;
    }
}

