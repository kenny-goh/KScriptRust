use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use colored::Colorize;

use crate::{Heap, Object, Opcode, Value};
use crate::callframe::CallFrame;
use crate::closure::{Closure, ObjUpvalue};
use crate::function::Function;
use crate::nativefn::{append_file_native, clock_native, NativeFn, NativeValue, str_native, write_file_native};

const MAX_CALLSTACK: usize = 256;
const MAX_VALUE_STACK: usize = 256;
const DEBUG: bool = true;

#[cfg(debug_assertions)]
macro_rules! log {
    ($( $args:expr ),*) => { /*println!( $( $args ),* ); */ }
}

// Non-debug version
#[cfg(not(debug_assertions))]
macro_rules! log {
    ($( $args:expr ),*) => {()}
}

/// Enum for run result
pub enum RunResult {
    Ok,
    RuntimeError,
}

/// Represent a virtual machine
///
pub struct VM {
    pub ip: usize,                                          // instruction pointer
    pub stack: Vec<Value>,                                  // Hold computation values
    pub shadow_stack: Vec<Value>,                           // Hold shadow values (removed for stack) for closing upvalues purpose
    pub callstack: Vec<CallFrame>,                          // List of call frames
    pub globals: HashMap<u32, Value>,                       // For global variables
    pub heap: Heap,                                         // For memory management (using Rust Box construct)
    pub curr_func_idx: usize,                               // For caching current function pointer
    pub open_upvalues: Option<Rc<RefCell<ObjUpvalue>>>      // For tracking open upvalues
    // pub _profile_duration: Duration                      // For testing
}

impl VM {
    /// Default constructor
    pub fn new() ->Self {
        VM {
            ip: 0,
            stack: vec![],
            shadow_stack: vec![Value::nil(); 256],
            callstack: vec![],
            globals: Default::default(),
            heap: Heap::new(),
            curr_func_idx: 0,
            open_upvalues: None
            // _profile_duration: Default::default()
        }
    }

    /// Reset the VM - for testing only!
    pub fn reset(&mut self) {
        self.ip = 0;
        self.stack.clear();
        self.globals.clear();
        self.heap.clear();
    }

    pub fn init(&mut self) {
        self.define_native("clock", clock_native);
        self.define_native("writeFile", write_file_native);
        self.define_native("appendFile", append_file_native);
        self.define_native("str", str_native);
    }

    /// Report run time error
    pub fn runtime_error(&mut self, message: &str) {
        println!("{} {}", "Runtime Error".bold().red(), message.bold().yellow());
        self.reset_stack();
    }

    /// Entry point to execute the virtual machine
    ///
    /// # Precondition
    /// 1. Init must have been called
    /// 2. The heap must contain results from the parsing phase.
    ///    eg String objects, Function objects, etc..
    pub fn execute(&mut self) -> RunResult {
        let func_main_idx = 0;  // Main function is always 0
        self.push(Value::object(Object::function(func_main_idx)));
        let upvalue_count = self.heap.get_mut_function(func_main_idx).upvalue_count;
        let closure_idx = self.new_closure(func_main_idx, upvalue_count);
        self.pop(); // Pop the function
        self.push(Value::Obj(Object::ClosureIndex(closure_idx)));
        self.call(closure_idx,0);
        return self.run();
    }

    /// Push value on to the stack
    fn push(&mut self, value: Value) {
        if self.stack.len()  >= MAX_VALUE_STACK {
            self.runtime_error("Stack overflow");
        }
        self.stack.push(value);
    }

    /// Pop value from the stack
    fn pop(&mut self)->Value {
        let pos = self.stack.len()-1;
        let val =  self.stack.pop().unwrap();
        self.shadow_stack[pos] = val;  // this incurred some performance overhead, presumably because of copy
        return val;
    }

    /// Run the VM
    fn run(&mut self)-> RunResult {

        let main_frame = self.callstack.last().unwrap();
        self.ip = main_frame.ip;
        self.curr_func_idx = self.heap.get_mut_closure(main_frame.closure_idx).func_idx;

        // The VM run loop
        loop {
            log!("LINE: {}", self.ip);
            log!("CALL STACK {:?}", &self.stack);
            let byte = self.read_byte();

            // Convert byte to opcode
            let opcode: Opcode = unsafe { std::mem::transmute(byte) };

            match opcode {
                Opcode::Constant => {
                    log!("OP CONSTANT");
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Opcode::Nil => {
                    log!("OP NIL");
                    self.push(Value::nil());
                }
                Opcode::True => {
                    log!("OP TRUE");
                    self.push(Value::bool(true));
                }
                Opcode::False => {
                    log!("OP FALSE");
                    self.push(Value::bool(false));
                }
                Opcode::Pop => {
                    log!("OP POP");
                    self.pop();
                }
                Opcode::DefineGlobal => {
                    log!("OP DEFINE GLOBAL VAR");
                    let str = self.read_string();
                    let str_hash = str.as_string_hash();
                    let value = *self.peek(0);
                    self.globals.insert(str_hash, value );
                    self.pop();
                }
                Opcode::GetGlobal => {
                    log!("OP GET GLOBAL VAR");
                    let str = self.read_string();
                    let str_hash = str.as_string_hash();
                    let option_value = self.globals.get(&str_hash);
                    let value = match option_value {
                        None => {
                            let message = format!("Undefined variable {}",
                                    self.heap.get_string(str_hash));
                            self.runtime_error(&*message);
                            return RunResult::RuntimeError
                        }
                        Some(content) => {
                            (*content).clone()
                        }
                    };
                    self.push(value);
                }
                Opcode::SetGlobal => {
                    log!("OP SET GLOBAL");
                    let str = self.read_string();
                    let str_hash = str.as_string_hash();
                    if self.globals.get(&str_hash).is_none() {
                        let message = format!("Undefined variable {}", self.heap.get_string(str_hash));
                        self.runtime_error(&message);
                        return RunResult::RuntimeError;
                    } else {
                        self.globals.insert(str_hash, *self.peek(0));
                    }
                }
                Opcode::GetLocal => {
                    log!("OP GET LOCAL");
                    let slot = self.read_byte() as usize;
                    log!("SLOT: {}", slot);
                    let slot_offset = self.callstack.last().unwrap().slot_offset;
                    log!("SLOT INDEX: {}", slot_offset);
                    let value = self.stack[slot + slot_offset];
                    log!("Value: {}", value);
                    self.push(value);
                }
                Opcode::SetLocal => {
                    log!("OP SET CONSTANT");
                    let slot = self.read_byte() as usize;
                    let slot_offset = self.callstack.last().unwrap().slot_offset;
                    self.stack[slot + slot_offset] = *self.peek(0);
                }
                Opcode::GetUpvalue => {
                    log!("OP GET UPVALUE");
                    // fixme: to test
                    let slot = self.read_byte();
                    let closure_idx = self.callstack.last().unwrap().closure_idx;
                    // fixme: this stinks
                    let location = self.heap.get_mut_closure(closure_idx)
                        .upvalues[slot as usize]
                        .as_ref()
                        .borrow_mut()
                        .resolve_value(&self);
                    self.push(location);
                }
                Opcode::SetUpvalue => {
                    log!("OP SET UPVALUE");
                    // fixme: to test
                    let slot = self.read_byte();
                    let closure_idx = self.callstack.last().unwrap().closure_idx;
                    // fixme: this stinks
                    self.heap.get_mut_closure(closure_idx)
                        .upvalues[slot as usize]
                        .as_ref()
                        .borrow_mut()
                        .location = Some((self.stack.len()-1) as usize);
                }
                Opcode::Equal => {
                    log!("OP EQUAL");
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::bool(a == b))
                }
                Opcode::Add => {
                    log!("OP ADD");
                    // fixme: refactor this to use self.bin_ops(..)
                    let b = *self.peek(0);
                    let a = *self.peek(1);
                    if Self::is_both_number(&a, &b) {
                        self.pop();
                        self.pop();
                        self.push(Value::number(a.as_number() + b.as_number()));
                    } else if Self::is_both_string(&a, &b) {
                        let str_b = self.heap.get_string(a.as_string_hash());
                        let str_a = self.heap.get_string(b.as_string_hash());

                        // Due to ownership rule, this is the easiest way to merge
                        // two borrowed strings
                        let mut merged = String::with_capacity(str_a.len() + str_b.len());
                        merged.push_str(&str_b);
                        merged.push_str(&str_a);

                        let hash = self.heap.alloc_string(merged);

                        // Bugfix: Garbage collection
                        // Make sure the newly allocated string is not garbage collected
                        // by pushing it to the stack and popping it off after gc check
                        self.push(Value::object(Object::string(hash)));
                        self.try_run_garbage_collection();
                        self.pop();
                        // End of Bugfix

                        self.pop();
                        self.pop();

                        self.push(Value::object(Object::string(hash)));
                    }
                    else {
                        self.runtime_error("Operands must be two numbers or two strings");
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Multiply => {
                    log!("OP MUL");
                    if !self.bin_ops(|a, b| a * b) {
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Divide => {
                    log!("OP DIV");
                    if !self.bin_ops(|a, b| a / b) {
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Subtract => {
                    log!("OP SUBS");
                    if !self.bin_ops(|a, b| a - b) {
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Less => {
                    log!("OP LESS");
                    if !self.bin_cmp(|a, b| a < b) {
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Greater => {
                    log!("OP GREATER");
                    if !self.bin_cmp(|a, b| a > b) {
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Negate => {
                    log!("OP NEGATE");
                    let value = self.pop();
                    if value.is_number() {
                        self.push(Value::number(-value.as_number()));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Not => {
                    log!("OP NOT");
                    let value = self.pop();
                    if value.is_boolean() {
                        self.push(Value::bool(!value.as_boolean()));
                    } else {
                        self.runtime_error("Operand must be a boolean.");
                        return RunResult::RuntimeError
                    }
                }
                Opcode::Jump => {
                    log!("OP JUMP");
                    let offset = self.read_short() as usize;
                    self.ip += offset;
                }
                Opcode::JumpIfFalse => {
                    log!("OP JUMP IF FALSE");
                    let offset = self.read_short() as usize;
                    let value = self.peek(0);
                    if !value.as_boolean() {
                        self.ip += offset
                    }
                }
                Opcode::Loop => {
                    log!("OP LOOP");
                    let offset = self.read_short() as usize;
                    self.ip -= offset;
                }
                Opcode::Call => {
                    log!("OP CALL");
                    let arg_count = self.read_byte();
                    let last = self.callstack.len()-1;
                    // Store current ip
                    self.callstack.get_mut(last).unwrap().ip = self.ip;
                    if !self.call_value(*self.peek(arg_count as usize), arg_count) {
                        return RunResult::RuntimeError;
                    }
                    let curr_frame = self.callstack.last().unwrap();
                    self.ip = curr_frame.ip;
                    // Cached the function ptr from the current callstack
                    self.curr_func_idx = self.heap.get_mut_closure(curr_frame.closure_idx).func_idx;
                }
                Opcode::Print => {
                    log!("OP PRINT");
                    let content = self.pop();
                    if content.is_string_hash() {
                        let hash = content.as_string_hash();
                        println!("{}", self.heap.get_string(hash));
                    } else {
                        println!("{}", content);
                    }
                }
                Opcode::Closure => {
                    log!("OP CLOSURE");
                    let func_idx = self.read_constant().as_function_index();
                    log!("FUNC: {}", self.heap.get_mut_function(func_idx).name);
                    let upvalue_count = self.heap.get_mut_function(func_idx).upvalue_count;
                    let closure_idx = self.new_closure(func_idx, upvalue_count);
                    self.push(Value::object(Object::ClosureIndex(closure_idx)));

                    //
                    let upvalues_count = self.heap.get_mut_closure(closure_idx).upvalues.len();
                    for i in 0..upvalues_count {
                        let is_local = self.read_byte();
                        let index = self.read_byte();

                        let curr_frame = self.callstack.last().unwrap();
                        if is_local == 1u8 {
                            // The upvalue is in local scope
                            let mut prev_upvalue: Option<Rc<RefCell<ObjUpvalue>>> = None;
                            let mut curr_upvalue = match &self.open_upvalues {
                                None => { None }
                                Some(it) => { Some(Rc::clone(&it)) }
                            };
                            let location = curr_frame.slot_offset + index as usize;
                            // todo: Untested path
                            while Self::upvalue_location_is_greater_than(&mut curr_upvalue, &location) {
                                // previous = current
                                prev_upvalue = Some(Rc::clone(&curr_upvalue.as_ref().unwrap()));
                                // current = current -> next
                                curr_upvalue = if Self::has_next(&mut curr_upvalue) {
                                    let next = Self::get_next(&mut curr_upvalue);
                                    Some(Rc::clone(next))
                                } else {
                                    None
                                }
                            }

                            if curr_upvalue.is_some() &&
                                curr_upvalue.as_ref().unwrap().as_ref().borrow().location.unwrap() == location {
                                self.heap.get_mut_closure(closure_idx).upvalues[i] = Rc::clone(&curr_upvalue.unwrap());
                            } else {
                                let mut next_link: Option<Rc<RefCell<ObjUpvalue>>> = None;
                                if curr_upvalue.is_some() {
                                    next_link = Some( Rc::clone(&curr_upvalue.unwrap()));
                                }
                                let created_upvalue = Rc::new(RefCell::new(
                                    ObjUpvalue::new(location, next_link )));

                                if prev_upvalue.is_none() {
                                    self.open_upvalues = Some(Rc::clone(&created_upvalue))
                                } else {
                                    // todo: Untested path
                                    unsafe {
                                        (*prev_upvalue.as_ref().unwrap().as_ptr()).next =
                                            Some(Rc::clone(&created_upvalue));
                                    }
                                }
                                self.heap.get_mut_closure(closure_idx).upvalues[i] = Rc::clone(&created_upvalue);
                            }
                        } else {
                            // The upvalue is in outer scope
                            let curr_frame_closure_idx = curr_frame.closure_idx;
                            self.heap.get_mut_closure(closure_idx).upvalues[i] = Rc::clone(
                                &self.heap.get_mut_closure(curr_frame_closure_idx).upvalues[index as usize]);
                        }
                    }
                }
                Opcode::CloseValue => {
                    self.pop();
                    self.close_upvalues(self.stack.len()-1);
                }
                Opcode::Return => {
                    log!("OP RETURN");

                    // Pop return value
                    let result = self.pop();
                    let frame_to_delete = self.callstack.pop().unwrap();
                    if self.callstack.is_empty() {
                         self.pop(); // Pop main function
                        // println!("profile duration is: {:?}", self._profile_duration);
                        return RunResult::Ok
                    }

                    // Discard call frame
                    let stack_len = self.stack.len();
                    for _ in frame_to_delete.slot_offset..stack_len {
                        self.pop();
                    }
                    self.close_upvalues(frame_to_delete.slot_offset);

                    // Push return value
                    self.push(result);

                    // Load the correct ip;
                    self.ip = self.callstack.last().unwrap().ip;

                    // Cached the function ptr from the current callstack
                    self.curr_func_idx = self.heap.get_mut_closure(self.callstack.last().unwrap().closure_idx).func_idx;
                }
            }
        }

    }

    fn upvalue_location_is_greater_than(upvalue: &mut Option<Rc<RefCell<ObjUpvalue>>>, location: &usize) -> bool {
        upvalue.is_some() &&
            upvalue.as_ref().unwrap().as_ref().borrow().location.as_ref().unwrap() > &location
    }

    fn get_next(upvalue: &mut Option<Rc<RefCell<ObjUpvalue>>>) -> &Rc<RefCell<ObjUpvalue>> {
        unsafe {
            return (*upvalue.as_ref().unwrap().as_ref().as_ptr()).next.as_ref().unwrap()
        }
    }

    fn has_next(upvalue: &mut Option<Rc<RefCell<ObjUpvalue>>>) -> bool {
        unsafe {
            return (*upvalue.as_ref().unwrap().as_ref().as_ptr()).next.is_some();
        }
    }

    fn new_closure(&mut self, func_idx: usize, upvalue_count: usize) -> usize {
        let mut closure = Closure::new(func_idx);
        closure.init_upvalues(upvalue_count);
        let closure_idx = self.heap.alloc_closure(closure);
        closure_idx
    }

    /// Run garbage collection if heap is ready for GC
    fn try_run_garbage_collection(&mut self) {
        if self.heap.is_ready_for_garbage_collection() {
            let mut marked_objects = vec![];
            self.mark_roots(&mut marked_objects);
            self.trace_references(&mut marked_objects);
            self.heap.run_gc(marked_objects);
        }
    }

    ///
    fn trace_references(&mut self, roots: &mut Vec<Value>) {
        for object in roots.clone() {
            match object {
                Value::Obj(object) => {
                    match object {
                        Object::FunctionIndex(idx) => {
                            for val in &self.heap.functions[idx].borrow_mut().chunk.constants {
                                roots.push(val.clone());
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    ///
    fn mark_roots(&mut self, roots: &mut Vec<Value>) {
        roots.extend(self.stack.clone());
        roots.extend(self.globals.values().cloned().collect::<Vec<Value>>());
        for str_hash in self.globals.keys() {
            roots.push(Value::Obj(Object::StringHash(*str_hash)))
        }
    }

    /// Shortcut for checking both strings are string hash
    fn is_both_string(a: &Value, b: &Value) -> bool {
        a.is_string_hash() && b.is_string_hash()
    }

    /// Shortcut for checking both strings are string hash
    fn is_both_number(a: &Value, b: &Value) -> bool {
        a.is_number() && b.is_number()
    }

    /// Interpret byte
    fn read_byte(&mut self)->u8 {
        unsafe {
            // Because curr_function is a pointer, * is needed to deference it
            let result = (*(self.curr_function())).chunk.code[self.ip];
            self.ip += 1;
            return result;
        }
    }

    /// Helper to get current function
    fn curr_function(&self) -> *mut Function {
        // performance optimization -> use pointer
        return self.heap.functions[self.curr_func_idx].as_ptr()
    }

    /// Interpret short (16 bit)
    fn read_short(&mut self)->u16 {
        // Unsafe due to use of ptr as performance optimization
        unsafe {
            let byte1 = (*(self.curr_function())).chunk.code[self.ip] as u16;
            let byte2 = (*(self.curr_function())).chunk.code[self.ip + 1] as u16;
            let result = (byte1 << 8 | byte2) as u16;
            self.ip += 2;
            return result;
        }
    }

    /// Interpret constant
    fn read_constant(&mut self) -> Value {
        // Unsafe due to use of ptr as performance optimization
        unsafe {
            let pos = self.read_byte() as usize;
            let value = (*(self.curr_function())).chunk.constants[pos];
            return value.clone();
        }
    }

    /// Interpret string
    fn read_string(&mut self) -> Object {
        let value = self.read_constant();
        return value.as_object().clone();
    }

    /// Peek stack based on the last position
    fn peek(&self, pos: usize) -> &Value {
        return self.stack.get(self.stack.len()-1-pos).unwrap();
    }

    /// Method to call a callable object. eg function, native function, instance method, etc..
    fn call_value(&mut self,
                  callee: Value,
                  arg_count: u8)->bool {
        if callee.is_closure_index() {
            let closure_idx = callee.as_closure_index();
            return self.call(closure_idx, arg_count);
        } else if callee.is_nativefn_index() {
            let native_fn_idx = callee.as_nativefn_index();
            return self.call_native(arg_count, native_fn_idx);
        }
        self.runtime_error("Can only call function and classes.");
        return false;
    }

    ///
    fn call_native(&mut self, arg_count: u8, native_fn_idx: usize) ->bool {
        let mut native_values: Vec<NativeValue> = vec![];
        self.convert_args_to_native(arg_count, &mut native_values);
        self.pop(); // pop function
        let native = self.heap.get_nativefn(native_fn_idx);
        let native_val: NativeValue = native(arg_count, native_values);
        let result = self.native_to_value(native_val);
        self.push(result);
        return true;
    }

    ///
    fn native_to_value(&mut self, native_val: NativeValue) -> Value {
        match native_val {
            NativeValue::String(s) => {
                let hash = self.heap.alloc_string(s);
                Value::Obj(Object::StringHash(hash))
            }
            NativeValue::Number(n) => Value::number(n),
            NativeValue::Boolean(b) => Value::Bool(b),
            NativeValue::Nil() => Value::nil()
        }
    }

    ///
    fn convert_args_to_native(&mut self, arg_count: u8, native_values: &mut Vec<NativeValue>) {
        for _ in 0..arg_count as usize {
            let value = self.pop();
            match value {
                Value::Number(n) => {
                    native_values.push(NativeValue::Number(n));
                }
                Value::Bool(b) => {
                    native_values.push(NativeValue::Boolean(b));
                }
                Value::Nil() => {
                    native_values.push(NativeValue::Nil());
                }
                Value::Obj(obj) => {
                    match obj {
                        Object::StringHash(hash) => {
                            let str = self.heap.get_string(hash).to_string();
                            native_values.insert(0, NativeValue::String(str));
                        }
                        _ => { panic!("Function, NativeFn are not allowed as argument to native function") }
                    }
                }
            }
        }
    }

    /// Insert the call into the call stack
    fn call(&mut self,
            closure_idx: usize,
            arg_count: u8) ->bool {
        let arity = self.heap.get_mut_function(self.heap.get_mut_closure(closure_idx).func_idx).arity;
        if arg_count as usize != arity {
            let message = format!("Expected {} arguments but got {}", arity, arg_count);
            self.runtime_error(&message);
        }
        let frame = CallFrame::new(closure_idx,
                                   self.stack.len() - 1 - arg_count as usize);
        self.callstack.push(frame);
        return true;
    }

    fn define_native(&mut self, name: &str, native: NativeFn) {
        let string_hash = self.heap.alloc_string(name.to_string());
        let native_fn_idx = self.heap.alloc_nativefn(native);
        self.globals.insert(string_hash, Value::Obj(Object::NativeFnIndex(native_fn_idx)));
    }

    /// Reset the stack
    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    /// Convenience method for binary operations
    fn bin_ops<F>(&mut self, mut apply: F) -> bool
        where F: FnMut(f64, f64)->f64 {

        let b = *self.peek(0);
        let a = *self.peek(1);

        if Self::is_both_number(&a, &b) {
            self.pop();
            self.pop();
            self.push(Value::number(apply(a.as_number(), b.as_number())));
        } else {
            self.runtime_error("Operands must be two numbers");
            return false;
        }
        return true;
    }

    /// Convenience method for binary comparisons
    fn bin_cmp<F>(&mut self, mut apply: F) -> bool
        where F: FnMut(Value,Value)->bool {

        let b = self.pop();
        let a = self.pop();

        if Self::is_both_number(&a, &b) {
            self.push(Value::bool(apply(a,b)));
        }
        else {
            self.runtime_error("Operands must be two numbers");
            return false;
        }
        return true;
    }

    fn close_upvalues(&mut self, frame_slot: usize) {
        while self.open_upvalues_location_greater_or_equal_to(&frame_slot) {
            let location = self.get_open_upvalues_location();
            let value = self.shadow_stack[location];
            self.close_upvalue(value);
            let next = if Self::has_next(&mut self.open_upvalues) {
                Some(Rc::clone(Self::get_next(&mut self.open_upvalues)))
            } else {
                None
            };
            self.open_upvalues = next;
            self.shadow_stack[location] = Value::Nil();
        }
    }

    fn close_upvalue(&mut self, value: Value) {
        unsafe {
            (*self.open_upvalues.as_ref().unwrap().as_ref().as_ptr()).closed = Some(value);
            (*self.open_upvalues.as_ref().unwrap().as_ref().as_ptr()).location = None;
        }
    }

    fn open_upvalues_location_greater_or_equal_to(&mut self, frame_slot: &usize) -> bool {
        self.open_upvalues.is_some() &&
            self.open_upvalues.as_ref().unwrap().as_ref().borrow().location.as_ref().unwrap() >= &frame_slot
    }

    fn get_open_upvalues_location(&mut self) -> usize {
        self.open_upvalues.as_ref().unwrap().as_ref().borrow().location.unwrap()
    }
}
