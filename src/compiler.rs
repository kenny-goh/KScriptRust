use std::collections::HashMap;
use std::{fmt, mem};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefMut;

use crate::function::{Function};
use crate::{Heap, Object, Opcode, Value};
use crate::closure::Upvalue;
use crate::token::{Token, TokenType};
use crate::debug::disassemble_chunk;

static DEBUG_MACHINE_CODE: bool = true;
static MAX_UPVALUE_COUNT: usize = 256;

#[derive(Copy, Clone)]
pub enum FunctionType {
    Main,
    Function,
    Method,
    Initializer,
}
impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            FunctionType::Main => {
                write!(f, "<Main>")
            }
            FunctionType::Function => {
                write!(f, "<Function>")
            }
            FunctionType::Method => {
                write!(f, "<Method>")
            }
            FunctionType::Initializer => {
                write!(f, "<Initializer>")
            }
        }
    }
}

/// Data structure for Local variable
#[repr(C)]
struct Local {
    name: String,
    depth: isize,
    pub is_captured: bool,
}

impl Local {
    pub fn from(name: String, depth: isize) -> Self {
        Local {
            name,
            depth,
            is_captured: false,
        }
    }
}

impl Clone for Local {
    fn clone(&self) -> Self {
        return Local::from(self.name.to_string(), self.depth);
    }
}


/// Data structure for Compiler
#[derive(Clone)]
#[repr(C)]
struct Compiler {
    /// Enclosing compiler index
    enclosing: usize,
    /// Which function is associated to this compiler
    function_idx: usize,
    /// Function type
    function_type: FunctionType,
    /// Keep track of scope
    scope_depth: isize,
    /// Keep track of local variables
    locals: Vec<Local>,
    /// Keep track of upvalues
    upvalues: Vec<Upvalue>
}


impl Compiler {
    pub fn new(enclosing: usize,
               function_idx: usize,
               function_type: FunctionType) -> Self {

        let local = match function_type {
            FunctionType::Method => Local::from("this".to_string(),0),
            FunctionType::Initializer => Local::from("this".to_string(),0),
            _ => Local::from("".to_string(),0)
        };
        Compiler {
            enclosing,
            function_idx,
            function_type,
            scope_depth: 0,
            locals: vec![local],
            upvalues: vec![]
        }
    }

    pub fn add_local(&mut self, name: String, depth: isize) {
        self.locals.push(Local::from(name, depth));
    }

    pub fn add_upvalues(&mut self, index: usize, is_local: bool) {
        self.upvalues.push(Upvalue::new(index, is_local));
    }

}


struct ClassCompiler {
    pub enclosing: Option<Box<ClassCompiler>>
}

impl ClassCompiler {
    pub fn new(enclosing: Option<Box<ClassCompiler>>) -> Self {
        ClassCompiler{
            enclosing
        }
    }

}

#[derive(Copy, Clone)]
#[derive(PartialEq, PartialOrd)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call
}

#[derive(Copy, Clone, Debug)]
enum ParseFn {
    None,
    Grouping,
    Call,
    Unary,
    Binary,
    Variable,
    String,
    Number,
    Literal,
    And,
    Or,
    Dot,
    This,
}

#[derive(Copy, Clone)]
struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

impl ParseRule {
    pub fn from(prefix: ParseFn,
                infix: ParseFn,
                precedence: Precedence) -> Self {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

/// Represent a parser that transform scanned tokens into
/// virtual machine code
pub struct Parser {
    /// Current index location
    curr_token_index: usize,
    panic_mode: bool,
    pub had_error: bool,
    /// List of compilers
    compilers: Vec<Compiler>,
    /// List of tokens
    tokens: Vec<Token>,
    /// Argument length of function
    function_arity: usize,
    /// Index to the compiler instances inside compilers
    curr_compiler_index: usize,
    current_class: Option<Box<ClassCompiler>>,
    /// For memory management using Rust Box construct
    pub heap: Heap,
    /// Parse rules for precedence based on Pratt algorithm
    parse_rules: HashMap<TokenType, ParseRule>,
}

impl Parser {
    pub fn new(heap: Heap,
               tokens: Vec<Token>) -> Self {
        Parser {
            curr_token_index: 0,
            panic_mode: false,
            had_error: false,
            compilers: vec![],
            tokens,
            function_arity: 0,
            curr_compiler_index: usize::MAX, // MAX means null
            current_class: None,
            heap,
            parse_rules: HashMap::from([
                (TokenType::LeftParen, ParseRule::from(ParseFn::Grouping, ParseFn::Call, Precedence::Call)),
                (TokenType::Dot, ParseRule::from(ParseFn::None, ParseFn::Dot, Precedence::Call)),
                (TokenType::Minus, ParseRule::from(ParseFn::Unary, ParseFn::Binary, Precedence::Term)),
                (TokenType::Plus, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Term)),
                (TokenType::Slash, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Factor)),
                (TokenType::Star, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Factor)),
                (TokenType::Bang, ParseRule::from(ParseFn::Unary, ParseFn::None, Precedence::None)),
                (TokenType::EqualEqual, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Equality)),
                (TokenType::BangEqual, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Equality)),
                (TokenType::Equal, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Equality)),
                (TokenType::Greater, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Comparison)),
                (TokenType::GreaterEqual, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Comparison)),
                (TokenType::Less, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Comparison)),
                (TokenType::LessEqual, ParseRule::from(ParseFn::None, ParseFn::Binary, Precedence::Comparison)),
                (TokenType::Identifier, ParseRule::from(ParseFn::Variable, ParseFn::None, Precedence::None)),
                (TokenType::String, ParseRule::from(ParseFn::String, ParseFn::None, Precedence::None)),
                (TokenType::Number, ParseRule::from(ParseFn::Number, ParseFn::None, Precedence::None)),
                (TokenType::And, ParseRule::from(ParseFn::None, ParseFn::And, Precedence::And)),
                (TokenType::Or, ParseRule::from(ParseFn::None, ParseFn::Or, Precedence::Or)),
                (TokenType::False, ParseRule::from(ParseFn::Literal, ParseFn::None, Precedence::None)),
                (TokenType::This, ParseRule::from(ParseFn::This, ParseFn::None, Precedence::None)),
                (TokenType::True, ParseRule::from(ParseFn::Literal, ParseFn::None, Precedence::None)),
                (TokenType::Nil, ParseRule::from(ParseFn::Literal, ParseFn::None, Precedence::None))
            ]),
        }
    }

    /// Compile the tokens into machine code.
    ///
    /// Returns the function pointer to main
    pub fn compile(&mut self) -> usize {

        let function_name = "main".to_string();
        let function = Function::new(function_name, self.function_arity);

        let main_func_idx = self.heap.alloc_function(function);

        let compiler = Compiler::new(usize::MAX, main_func_idx, FunctionType::Main);
        self.curr_compiler_index = self.compilers.len();
        self.compilers.push(compiler);

        while !self.is_at_end() {
            self.declaration()
        }

        return self.end_compiler();
    }

    /// Begin a new scope
    fn begin_scope(&mut self) {
        let index = self.curr_compiler_index as usize;
        self.compilers[index].scope_depth += 1;
    }

    /// End the current scope
    fn end_scope(&mut self) {
        let index = self.curr_compiler_index as usize;
        self.compilers[index].scope_depth -= 1;
        let mut curr_local_count = self.current_compiler().locals.len();

        while curr_local_count > 0 &&
            self.current_compiler().locals[curr_local_count-1].depth > self.current_scope_depth() {
            let local_idx = self.current_compiler().locals.len() - 1;
            if (self.current_compiler().locals[local_idx]).is_captured {
                self.emit_byte(Opcode::CloseValue.byte());
            } else {
                // Pop the current scope
                self.emit_byte(Opcode::Pop.byte());
            }
            // Pop the current local variable
            self.compilers[self.curr_compiler_index as usize].locals.pop();

            curr_local_count = self.current_compiler().locals.len();
        }
    }
    
    /// Ends the current compiler
    fn end_compiler(&mut self) -> usize {
        self.emit_return();

        let func_index = self.compilers[self.curr_compiler_index as usize].function_idx;
        let chunk = self.heap.get_mut_function(func_index).chunk.clone();

        if !self.had_error {
            if DEBUG_MACHINE_CODE {
                disassemble_chunk(&chunk, &self.heap, &self.current_function().name);
            }
        }

        let enclosing = self.compilers[self.curr_compiler_index as usize].enclosing;
        self.curr_compiler_index = enclosing;

        return func_index;
    }

    /// Check if the current token match the given token type
    /// Note: this call does not consume the token
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        let result = self.peek().token_type == token_type;
        return result;
    }

    /// Peek the current token
    fn peek(&self) -> Token {
        return self.tokens.get(self.curr_token_index).unwrap().clone();
    }

    /// Are we at EOF yet?
    fn is_at_end(&self) -> bool {
        return self.peek().token_type == TokenType::Eof;
    }

    /// Move to the next token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr_token_index += 1;
        }
        return self.previous();
    }

    /// Retrieve the previous token
    fn previous(&self) -> Token {
        self.tokens.get(self.curr_token_index - 1).unwrap().clone()
    }

    /// Eat the current token
    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.peek().token_type == token_type {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    /// Report error at current token
    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.peek(), message);
    }

    /// Report error at previous token
    fn error(&mut self, message: &str) {
        self.error_at(self.previous(), message);
    }

    /// Helper method to report error
    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error ", token.line);
        if token.token_type == TokenType::Eof {
            eprintln!("at end ");
        } else if token.token_type == TokenType::Error {
            // do nothing
        } else {
            eprintln!("at {}", token.literal)
        }
        eprintln!("{}", message);
        self.had_error = true;
    }

    /// Helper method to retrieve current function as mutable
    fn current_function(&self) -> RefMut<Function> {
        let fn_hash = &self.compilers[self.curr_compiler_index as usize].function_idx;
        self.heap.functions[*fn_hash].borrow_mut()
    }

    /// Write 1 byte to the current function chunk
    fn emit_byte(&mut self, byte: u8) {
        self.current_function().chunk.code(byte, self.previous().line);
    }

    /// Write 2 bytes to the current function chunk
    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    /// Shortcut for writing return statement to function chunk
    fn emit_return(&mut self) {
        match self.current_compiler().function_type {
            FunctionType::Initializer => {
                self.emit_bytes(Opcode::GetLocal.byte(), 0);
            }
            _ => {
                self.emit_byte(Opcode::Nil.byte());
            }
        }
        self.emit_byte(Opcode::Return.byte());
    }

    /// Shortcut for writing a jump instruction to function chunk
    fn emit_jump(&mut self, instruction: u8) -> u8 {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        return (self.current_function().chunk.code.len() - 2) as u8;
    }

    /// Shortcut for writing constant to function chunk
    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(Opcode::Constant as u8, constant);
    }

    /// Shortcut for writing loop statement to function chunk
    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(Opcode::Loop.byte());
        let offset = self.current_function().chunk.code.len() - loop_start + 2;
        if offset >= 65536 {
            self.error("Loop body too large");
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8)
    }

    /// Short cut for patching current jump location to the given offset
    fn patch_jump(&mut self, offset: usize) {
        let jump = (self.current_function().chunk.code.len() - offset - 2) as u16;
        if jump >= 65535 {
            self.error("Too much code to jump over");
        }
        self.current_function().chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_function().chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    fn match_token_type(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        return true;
    }

    fn declaration(&mut self) {
        if self.match_token_type(TokenType::Fun) {
            self.fun_declaration();
        } else if self.match_token_type(TokenType::Var) {
            self.var_declaration();
        } else if self.match_token_type(TokenType::Class) {
           self.class_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect a function name");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn function(&mut self, function_type: FunctionType) {

        let function_name = match function_type {
            FunctionType::Main => "Main".to_string(),
            _ => self.previous().lexeme.to_string(),
        };

        let function = Function::new(function_name, self.function_arity);
        let func_idx = self.heap.alloc_function(function);

        // Start of new compiler
        let compiler = Compiler::new(self.curr_compiler_index, func_idx, function_type);
        self.curr_compiler_index = self.compilers.len();
        self.compilers.push(compiler);
        let compiler_idx = self.compilers.len()-1;

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name");
        if !self.check(TokenType::RightParen) {
            loop {
                self.current_function().arity += 1;
                if self.current_function().arity >= 255 {
                    self.error_at_current("Can't have more than 255 parameters");
                }
                let constant = self.parse_variable("Expect a parameter name");
                self.define_variable(constant);
                if !self.match_token_type(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body");
        self.block();

        self.end_compiler();

        let constant = self.make_constant(Value::Obj(Object::FunctionIndex(func_idx)));
        // self.emit_bytes(Opcode::Constant.byte(), constant );
        self.emit_bytes(Opcode::Closure.byte(), constant);

        let mut upvalue_count = 0;
        if self.heap.functions[func_idx].borrow().upvalue_count > 0 {
            upvalue_count = self.heap.functions[func_idx].borrow().upvalue_count;
        }
        for i in 0..upvalue_count {
            let is_local = self.compilers[compiler_idx].upvalues[i].is_local;
            let upvalue_index_byte = self.compilers[compiler_idx].upvalues[i].index as u8;
            if is_local {
                self.emit_byte(1u8);
            } else {
                self.emit_byte(0u8);
            }
            self.emit_byte(upvalue_index_byte);
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect a variable name.");
        if self.match_token_type(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(Opcode::Nil as u8)
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn define_variable(&mut self, global: u8) {
        if self.current_scope_depth() > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(Opcode::DefineGlobal as u8, global)
    }

    fn mark_initialized(&mut self) {
        let index = self.curr_compiler_index as usize;
        let locals_len = self.compilers[index].locals.len();
        if locals_len > 0 {
            self.compilers[index].locals[locals_len-1].depth = self.current_scope_depth();
        }
    }

    fn current_scope_depth(&mut self) -> isize {
        self.compilers[self.curr_compiler_index as usize].scope_depth
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);
        self.declare_variable();
        if self.current_scope_depth() > 0 {
            return 0;
        };
        return self.identifier_constant(&self.previous().lexeme);
    }

    fn declare_variable(&mut self) {
        if self.current_scope_depth() == 0 {
            return;
        }
        let name = &self.previous().lexeme;
        let current_scope_depth = self.current_scope_depth();

        for i in (0..self.current_compiler().locals.len()).rev() {
            let local = &self.current_compiler().locals[i];
            if local.depth != -1 && local.depth < current_scope_depth {
                break;
            }
            if *name == local.name {
                self.error("Already a variable of this name in this scope");
            }
        }
        self.compilers[self.curr_compiler_index as usize].add_local(name.to_string(), -1);
    }

    fn current_compiler(&mut self) -> &Compiler {
        return &self.compilers[self.curr_compiler_index as usize];
    }

    fn identifier_constant(&mut self, token_name: &str) -> u8 {
        let string_hash = self.heap.alloc_string(token_name.to_string());
        return self.make_constant( Value::object(Object::string(string_hash)));
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant_index = self.current_function().chunk.add_constants(value);
        if constant_index >= 255 {
            self.error_at_current("Too many constants in one chunk");
        }
        return constant_index;
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        self.advance();
        while !self.is_at_end() {
            if matches!(self.previous().token_type, TokenType::Semicolon) {
                return;
            }
            match self.peek().token_type {
                TokenType::Class | TokenType::For    | TokenType::Fun | TokenType::If |
                TokenType::Print | TokenType::Return | TokenType::Var |
                TokenType::While  => { return; }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // fixme: This can go into infinite loop when there is an error with parsing inside this function
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let prefix_rule_option = self.parse_rules.get(&self.previous().token_type);
        let mut prefix_rule: Option<ParseFn> = Some(ParseFn::None);
        if prefix_rule_option.is_some() {
            prefix_rule = Some(prefix_rule_option.unwrap().prefix);
        }

        let can_assign = precedence <= Precedence::Assignment;

        if self.call_rule_function(&mut prefix_rule, can_assign) == false {
            return;
        }

        loop {
            let parse_rule_option = self.parse_rules.get(&self.peek().token_type);
            let mut precedence_rule: Option<Precedence> = Some(Precedence::None);
            if parse_rule_option.is_some() {
                precedence_rule = Some(parse_rule_option.unwrap().precedence);
            }

            if precedence > precedence_rule.unwrap() {
                break;
            }
            self.advance();
            let infix_rule_option = self.parse_rules.get(&self.previous().token_type);
            let mut infix_rule: Option<ParseFn> = Some(ParseFn::None);
            if infix_rule_option.is_some() {
                infix_rule = Some(infix_rule_option.unwrap().infix);
            }
            if self.call_rule_function(&mut infix_rule, can_assign) == false {
                return;
            }
        }

        if can_assign && self.match_token_type(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn call_rule_function(&mut self, prefix_rule: &mut Option<ParseFn>, can_assign: bool) -> bool {
        match prefix_rule.unwrap() {
            ParseFn::None => {
                self.error("Expect expression");
                return false;
            }
            ParseFn::Grouping => self.grouping(),
            ParseFn::Call => self.call(),
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::String => self.string(),
            ParseFn::Number => self.number(),
            ParseFn::Literal => self.literal(),
            ParseFn::And => self.and(),
            ParseFn::Or => self.or(),
            ParseFn::Dot => self.dot(can_assign),
            ParseFn::This => self.this()
        }
        return true;
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn string(&mut self) {
        let string_hash = self.heap.alloc_string(self.previous().literal);
        self.emit_constant(Value::object(Object::StringHash(string_hash)));
    }

    fn number(&mut self) {
        let value: f64 = self.previous().lexeme.parse().unwrap();
        self.emit_constant(Value::number(value));
    }

    fn literal(&mut self) {
        match self.previous().token_type {
            TokenType::False => { self.emit_byte(Opcode::False as u8); }
            TokenType::True => { self.emit_byte(Opcode::True as u8); }
            TokenType::Nil => { self.emit_byte(Opcode::Nil as u8); }
            _ => {
                return; // unreachable
            }
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous(), can_assign);
    }

    fn statement(&mut self) {
        if self.match_token_type(TokenType::Print) {
            self.print_statement();
        } else if self.match_token_type(TokenType::For) {
            self.for_statement();
        } else if self.match_token_type(TokenType::If) {
            self.if_statement();
        } else if self.match_token_type(TokenType::Return) {
            self.return_statement();
        } else if self.match_token_type(TokenType::While) {
            self.while_statement();
        } else if self.match_token_type(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_function().chunk.code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after while.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
        let exit_jump = self.emit_jump(Opcode::JumpIfFalse.byte());
        self.emit_byte(Opcode::Pop.byte());
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump as usize);
        self.emit_byte(Opcode::Pop.byte());
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after if.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(Opcode::JumpIfFalse.byte());
        self.emit_byte(Opcode::Pop.byte());
        self.statement();

        let else_jump = self.emit_jump(Opcode::Jump.byte());
        self.patch_jump(then_jump as usize);
        self.emit_byte(Opcode::Pop.byte());

        if self.match_token_type(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump as usize);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.match_token_type(TokenType::Semicolon) {
            // No initializer
        } else if self.match_token_type(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_function().chunk.code.len();
        let mut exit_jump: isize = -1;

        if !self.match_token_type(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition");

            // Jump out of the loop if condition is false
            exit_jump = self.emit_jump(Opcode::JumpIfFalse.byte()) as isize;
            self.emit_byte(Opcode::Pop.byte());
        }

        if !self.match_token_type(TokenType::RightParen) {
            let body_jump = self.emit_jump(Opcode::Jump.byte());
            let increment_start = self.current_function().chunk.code.len();
            self.expression();
            self.emit_byte(Opcode::Pop.byte());
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump as usize);
        }

        self.statement();

        self.emit_loop(loop_start);

        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_byte(Opcode::Pop.byte());
        }

        self.end_scope();
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(Opcode::Pop as u8)
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(Opcode::Print as u8);
    }

    fn binary(&mut self) {
        let prefix_rule_option = self.parse_rules.get(&self.previous().token_type);
        if prefix_rule_option.is_some() {
            let prev = self.previous();
            let prec = prefix_rule_option.unwrap().precedence as u8;
            let next_prec: Precedence = unsafe { mem::transmute(prec + 1u8) };
            self.parse_precedence(next_prec);
            match prev.token_type {
                TokenType::Plus => self.emit_byte(Opcode::Add.byte()),
                TokenType::Star => self.emit_byte(Opcode::Multiply.byte()),
                TokenType::Slash => self.emit_byte(Opcode::Divide.byte()),
                TokenType::Minus => self.emit_byte(Opcode::Subtract.byte()),
                TokenType::BangEqual => self.emit_bytes(Opcode::Equal.byte(), Opcode::Not.byte()),
                TokenType::EqualEqual => self.emit_byte(Opcode::Equal.byte()),
                TokenType::Less => self.emit_byte(Opcode::Less.byte()),
                TokenType::LessEqual => self.emit_bytes(Opcode::Greater.byte(), Opcode::Not.byte()),
                TokenType::Greater => self.emit_byte(Opcode::Greater.byte()),
                TokenType::GreaterEqual => self.emit_bytes(Opcode::Less.byte(), Opcode::Not.byte()),
                _ => {
                    panic!("Unreachable code");
                }
            }
        } else {
            panic!("Unreachable code");
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.previous().token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(Opcode::Negate.byte()),
            TokenType::Bang => self.emit_byte(Opcode::Not.byte()),
            _ => { return; }
        }
    }

    fn dot(&mut self, can_assign: bool) {
        self.consume(TokenType::Identifier, "Expect field name after '.'.");
        let name = self.identifier_constant(&self.previous().lexeme);
        if can_assign && self.match_token_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(Opcode::SetProperty.byte(), name);
        } else if self.match_token_type(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.emit_bytes(Opcode::Invoke.byte(), name);
            self.emit_byte(arg_count);
        }
        else {
            self.emit_bytes(Opcode::GetProperty.byte(), name);
        }
    }


    fn and(&mut self) {
        let end_jump = self.emit_jump(Opcode::JumpIfFalse.byte());
        self.emit_byte(Opcode::Pop.byte());
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump as usize);
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(Opcode::JumpIfFalse.byte());
        let end_jump = self.emit_jump(Opcode::Jump.byte());
        self.patch_jump(else_jump as usize);
        self.emit_byte(Opcode::Pop.byte());
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump as usize);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) &&
            !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn return_statement(&mut self) {
        if self.current_function().name == "main" {
            self.error("Can't return from main.");
        }
        else {
            if self.match_token_type(TokenType::Semicolon) {
                self.emit_return();
            } else {
                match self.current_compiler().function_type {
                    FunctionType::Initializer => {self.error("Can't return value from an initializer.")}
                    _ => {}
                }
                self.expression();
                self.consume(TokenType::Semicolon, "Expect ';' after return value.");
                self.emit_byte(Opcode::Return.byte());
            }
        }
    }

    fn call(&mut self) {
        // fixme: implement me
        let arg_count = self.argument_list();
        self.emit_bytes(Opcode::Call.byte(), arg_count);
    }

    fn argument_list(&mut self)->u8 {
        let mut arg_count:u8 = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;
                if !self.match_token_type(TokenType::Comma) { break; }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments");
        return arg_count;
    }

    fn named_variable(&mut self, token: Token, can_assign: bool) {

        let mut set_op: u8 = Opcode::SetGlobal.byte();
        let mut get_op: u8 = Opcode::GetGlobal.byte();

        let current_compiler_index = self.curr_compiler_index as usize;

        let mut arg = self.resolve_local(current_compiler_index, &token);
        if arg != usize::MAX {
            set_op = Opcode::SetLocal.byte();
            get_op = Opcode::GetLocal.byte();
        } else {
            arg = self.resolve_upvalue(current_compiler_index, &token);
            if arg != usize::MAX {
                set_op = Opcode::SetUpvalue.byte();
                get_op = Opcode::GetUpvalue.byte();
            }
            else {
                arg = self.identifier_constant(&token.lexeme) as usize;
            }
        }

        if can_assign && self.match_token_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op, arg as u8);
        } else if can_assign && self.match_token_type(TokenType::PlusEqual) {
            self.emit_bytes(get_op, arg as u8);
            self.expression();
            self.emit_byte(Opcode::Add.byte());
            self.emit_bytes(set_op, arg as u8);
        } else if can_assign && self.match_token_type(TokenType::MinusEqual) {
            self.emit_bytes(get_op, arg as u8);
            self.expression();
            self.emit_byte(Opcode::Subtract.byte());
            self.emit_bytes(set_op, arg as u8);
        } else {
            self.emit_bytes(get_op, arg as u8);
        }
    }

    fn resolve_local(&mut self, compiler_idx: usize, token: &Token) -> usize {
        let compiler = &self.compilers[compiler_idx];
        for i in (0..compiler.locals.len()).rev() {
            let local = &compiler.locals[i];
            if token.lexeme == local.name {
                if local.depth == -1 {
                    self.error("Can't read a local variable in its own initializer.")
                }
                return i;
            }
        }
        return usize::MAX;
    }

    fn add_upvalue(&mut self, compiler_idx:usize, index: usize, is_local: bool)->usize {
        let function_idx = self.compilers[compiler_idx].function_idx;
        let upvalue_count = self.heap.get_mut_function(function_idx).upvalue_count;
        for i in 0..upvalue_count {
            let upvalue_is_local = self.compilers[compiler_idx].upvalues[i].is_local;
            let upvalue_index   = self.compilers[compiler_idx].upvalues[i].index;
            if upvalue_index == index && upvalue_is_local == is_local {
                return i;
            }
        }
        if upvalue_count == MAX_UPVALUE_COUNT {
            self.error("Too many closures in function.");
            return 0;
        }
        self.heap.get_mut_function(function_idx).upvalue_count+= 1;
        self.compilers[compiler_idx].add_upvalues(index, is_local);
        return upvalue_count;
    }

    fn resolve_upvalue(&mut self, compiler_idx: usize,  name: &Token)->usize {
        let enclosing_idx = self.compilers[compiler_idx].enclosing;
        if enclosing_idx == usize::MAX {
            return usize::MAX;
        }

        let local = self.resolve_local(enclosing_idx as usize, name);
        if local != usize::MAX {
            self.compilers[enclosing_idx as usize].locals[local as usize].is_captured = true;
            return self.add_upvalue(compiler_idx, local as usize, true);
        }

        let upvalue = self.resolve_upvalue(enclosing_idx as usize, name);
        if upvalue != usize::MAX {
            return self.add_upvalue(compiler_idx, upvalue as usize, false);
        }

        return usize::MAX;
    }


    fn class_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect a class name.");
        let class_name = self.previous();
        let name_constant = self.identifier_constant(&self.previous().lexeme);
        self.declare_variable();

        self.emit_bytes(Opcode::Class.byte(), name_constant);
        self.define_variable(name_constant);

        let mut class_compiler = Some(Box::new(ClassCompiler::new(self.current_class.take())));
        self.current_class = class_compiler;

        self.named_variable(class_name, false);

        self.consume(TokenType::LeftBrace, "Expect '{' before class body");
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.method();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.");
        self.emit_byte(Opcode::Pop.byte()); // pop class name


        self.current_class = self.enclosing_class()
    }

    fn enclosing_class(&mut self) -> Option<Box<ClassCompiler>> {
        match self.current_class.take() {
            Some(it) => {
                match it.enclosing {
                    Some(it2) => { Some(Box::new(*it2)) }
                    None => None
                }
            },
            None => None,
        }
    }

    fn method(&mut self) {
        self.consume(TokenType::Identifier, "Expect a method name.");
        let constant = self.identifier_constant(&self.previous().lexeme);
        let func_type = if self.previous().lexeme == "init" {
            FunctionType::Initializer
        } else {
            FunctionType::Method
        };
        self.function(func_type);
        self.emit_bytes(Opcode::Method.byte(), constant);
    }

    fn this(&mut self) {
        if (self.current_class.is_none()) {
            self.error("Can't use 'this' outside of class");
            return;
        }
        self.variable(false);
    }
}

