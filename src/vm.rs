use std::borrow::{Borrow};
use std::cell::RefCell;
use std::rc::Rc;
use colored::Colorize;
use fnv::{ FnvHashMap};

use crate::{Heap, Object, Opcode, Value};
use crate::callframe::CallFrame;
use crate::class::{BoundMethod, Class, Instance};
use crate::closure::{Closure, ObjUpvalue};
use crate::function::Function;
use crate::nativefn::{append_file_native, clock_native, NativeFn, NativeValue, str_native, write_file_native};

const CHECK_GC_INTERVAL: usize =  5000;
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

// fixme: Too many conversion e.g usize,

/// Represent a virtual machine
///
pub struct VM {
    pub ip: usize,                                          // instruction pointer
    pub stack: Vec<Value>,                                  // Hold computation values
    pub callstack: Vec<CallFrame>,                          // List of call frames
    pub globals: FnvHashMap<u32, Value>,
    pub heap: Heap,                                         // For memory management (using Rust Box construct)
    pub curr_func_idx: usize,                               // For caching current function pointer
    pub open_upvalues: Option<Rc<RefCell<ObjUpvalue>>>,      // For tracking open upvalues
    pub stack_top: usize,
    pub init_string_hash: u32,
    // pub _profile_duration: Duration                      // For testing
}

impl VM {
    /// Default constructor
    pub fn new() ->Self {
        VM {
            ip: 0,
            stack: vec![Value::Nil();256],
            callstack: Vec::with_capacity(256),
            globals: FnvHashMap::default(),
            heap: Heap::new(),
            curr_func_idx: 0,
            open_upvalues: None,
            stack_top: 0,
            init_string_hash: 0
            // _profile_duration: Default::default()
        }
    }

    /// Reset the VM - for testing only!
    pub fn reset(&mut self) {
        self.ip = 0;
        self.stack.clear();
        self.globals.clear();
        self.heap.clear();
        self.curr_func_idx = 0;
        self.open_upvalues = None;
        self.stack_top = 0;
    }

    pub fn init(&mut self) {
        self.define_native("clock", clock_native);
        self.define_native("writeFile", write_file_native);
        self.define_native("appendFile", append_file_native);
        self.define_native("str", str_native);
        self.init_string_hash = self.heap.alloc_string("init".to_string());
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
        let upvalue_count = self.heap.get_function(func_main_idx).upvalue_count;
        let closure_idx = self.new_closure(func_main_idx, upvalue_count);
        self.fpop(); // Pop the function
        self.push(Value::Obj(Object::ClosureIndex(closure_idx)));
        self.call(closure_idx,0);
        return self.run();
    }

    /// Push value on to the stack
    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    /// Pop value from the stack
    #[inline(always)]
    fn pop(&mut self)->Value {
        self.stack_top -= 1;
        return self.stack.get(self.stack_top).unwrap().clone();
    }

    /// Fast pop without returning value
    #[inline(always)]
    fn fpop(&mut self) {
        self.stack_top -= 1;
    }

    /// Run the VM
    fn run(&mut self)-> RunResult {

        let main_frame = self.callstack.last().unwrap();

        let mut ip_counter = 0;
        self.ip = main_frame.ip;
        self.curr_func_idx = self.heap.get_closure(main_frame.closure_idx).func_idx;

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
                    self.fpop();
                }
                Opcode::DefineGlobal => {
                    log!("OP DEFINE GLOBAL VAR");
                    let str = self.read_string();
                    let str_hash = str.as_string_hash();
                    let value = *self.peek(0);
                    self.globals.insert(str_hash, value );
                    self.fpop();
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
                        Some(content) => (*content)
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
                    let slot = self.read_byte();
                    let closure_idx = self.callstack.last().unwrap().closure_idx;
                    let value = self.resolve_upvalue_location(slot, closure_idx);
                    self.push(value);
                }
                Opcode::SetUpvalue => {
                    log!("OP SET UPVALUE");
                    let slot = self.read_byte();
                    let closure_idx = self.callstack.last().unwrap().closure_idx;
                    self.set_upvalue_location(slot, closure_idx);
                }
                Opcode::GetProperty => {
                    let instance_idx = self.peek(0).as_instance_index();
                    let field_name_hash = self.read_string().as_string_hash();
                    let field_name = self.heap.get_string(field_name_hash).clone();
                    let class_idx = self.heap.get_instance(instance_idx).class_idx;
                    if self.heap.get_instance(instance_idx).fields.contains_key(&field_name_hash) {
                        let value = self.heap.get_instance(instance_idx).fields.get(&field_name_hash).unwrap().clone();
                        self.fpop(); // instance
                        self.push(value);
                    }
                    else if !self.bind_method(class_idx, field_name_hash) {
                        let error_text = format!("Undefined property {}", field_name);
                        self.runtime_error( &error_text );
                        return RunResult::RuntimeError;
                    }
                }
                Opcode::SetProperty => {
                    if !self.peek(1).is_instance_index() {
                        self.runtime_error("Only instance have fields.");
                        return RunResult::RuntimeError;
                    }
                    let instance_idx = self.peek(1).as_instance_index();
                    let field_name_hash = self.read_string().as_string_hash();
                    self.heap.get_mut_instance(instance_idx).fields.insert(field_name_hash, *self.peek(0) );
                    let value = self.pop();
                    self.fpop(); // instance
                    self.push(value)
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
                        self.fpop();
                        self.fpop();
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

                        self.fpop();
                        self.fpop();

                        self.push(Value::object(Object::string(hash)));
                    }
                    else {
                        self.runtime_error("Operands must be numbers or two strings");
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
                    let arg_count = self.read_byte() as usize;
                    let curr_callstack = self.callstack.len()-1;
                    // Store current ip
                    self.callstack.get_mut(curr_callstack).unwrap().ip = self.ip;
                    if !self.call_value(*self.peek(arg_count ), arg_count) {
                        return RunResult::RuntimeError;
                    }
                    let curr_frame = self.callstack.last().unwrap();
                    self.ip = curr_frame.ip;
                    // Cached the function ptr from the current callstack
                    self.curr_func_idx = self.heap.get_closure(curr_frame.closure_idx).func_idx;
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
                    log!("FUNC: {}", self.heap.get_function(func_idx).name);
                    let upvalue_count = self.heap.get_function(func_idx).upvalue_count;
                    let closure_idx = self.new_closure(func_idx, upvalue_count);
                    self.push(Value::object(Object::ClosureIndex(closure_idx)));

                    //
                    let upvalues_count = self.heap.get_closure(closure_idx).upvalues.len();
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
                            while Self::upvalue_location_is_greater_than(&curr_upvalue, &location) {
                                // previous = current
                                prev_upvalue = Some(Rc::clone(&curr_upvalue.as_ref().unwrap()));
                                // current = current -> next
                                curr_upvalue = if Self::has_next_upvalue(&mut curr_upvalue) {
                                    Self::get_next_upvalue(&curr_upvalue)
                                } else {
                                    None
                                }
                            }

                            if Self::upvalue_location_match(&curr_upvalue, location) {
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
                    self.fpop();
                    self.close_upvalues(self.stack_top-1);
                }
                Opcode::Class => {
                    let str_hash = self.read_constant().as_string_hash();
                    let class_name = self.heap.get_string(str_hash);
                    let class = Class::new(class_name.to_string());
                    let class_idx = self.heap.alloc_class(class);
                    self.push(Value::Obj(Object::ClassIndex(class_idx)));
                }
                Opcode::Method => {
                    log!("OP METHOD");
                    let string_hash = self.read_string().as_string_hash();
                    self.define_method(string_hash);
                }
                Opcode::Return => {
                    log!("OP RETURN");

                    // Pop return value
                    let result = self.pop();
                    let frame_to_delete = self.callstack.pop().unwrap();
                    if self.callstack.is_empty() {
                         self.fpop(); // Pop main function
                        // println!("profile duration is: {:?}", self._profile_duration);
                        return RunResult::Ok
                    }

                    // Discard call frame
                    let stack_len = self.stack_top;
                    for _ in frame_to_delete.slot_offset..stack_len {
                        self.fpop(); // performance-tuning
                    }
                    self.close_upvalues(frame_to_delete.slot_offset);

                    // Push return value
                    self.push(result);

                    // Load the correct ip;
                    self.ip = self.callstack.last().unwrap().ip;

                    // Cached the function ptr from the current callstack
                    self.curr_func_idx = self.heap.get_closure(self.callstack.last().unwrap().closure_idx).func_idx;
                }
            }

            if ip_counter % CHECK_GC_INTERVAL == 0 {
                self.try_run_garbage_collection();
            }

            ip_counter += 1;
        }

    }

    fn set_upvalue_location(&mut self, slot: u8, closure_idx: usize) {
        self.heap.get_closure(closure_idx)
            .upvalues[slot as usize]
            .as_ref().borrow_mut()
            .location = Some((self.stack_top - 1) as usize);
    }

    fn resolve_upvalue_location(&mut self, slot: u8, closure_idx: usize) -> Value {
        let location = self.heap.get_closure(closure_idx)
            .upvalues[slot as usize]
            .as_ref()
            .borrow_mut()
            .resolve_value(&self);
        location
    }

    fn upvalue_location_match(curr_upvalue: &Option<Rc<RefCell<ObjUpvalue>>>, location: usize) -> bool {
        match curr_upvalue {
            Some(it) => it.as_ref().borrow().location == Some(location),
            None => false
        }
    }

    fn upvalue_location_is_greater_than(upvalue: &Option<Rc<RefCell<ObjUpvalue>>>, location: &usize) -> bool {
        match upvalue {
            None => false,
            Some(it) => it.as_ref().borrow().location > Some(*location)
        }
    }

    fn get_next_upvalue(upvalue: &Option<Rc<RefCell<ObjUpvalue>>>) -> Option<Rc<RefCell<ObjUpvalue>>> {
        return Some(Rc::clone(&upvalue.as_ref().unwrap()              // unwrap option of RC without dropping
                                            .as_ref().borrow().next         // access upvalue object inside refcell
                                            .as_ref().unwrap()));           // unwrap option of next without dropping
    }

    fn has_next_upvalue(upvalue: &mut Option<Rc<RefCell<ObjUpvalue>>>) -> bool {
        match upvalue {
            Some(it) => it.as_ref().borrow_mut().next.is_some(),
            None => false
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
            // fixme: mark class and instances
            self.mark_roots(&mut marked_objects);
            // fixme: trace references under class and instances
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
                        Object::ClosureIndex(idx) => {
                            let func_dx = self.heap.get_closure(idx).func_idx;
                            // Constants
                            for val in &self.heap.functions[func_dx].borrow().chunk.constants {
                                roots.push(val.clone());
                            }
                            // Function
                            roots.push(Value::Obj(Object::FunctionIndex(func_dx)));
                            // Upvalues that have been closed
                            for val in &self.heap.get_closure(idx).upvalues {
                                if !val.as_ref().borrow().is_null {
                                    match val.as_ref().borrow().closed {
                                        Some(it) => roots.push(it.clone()),
                                        None => {}
                                    }
                                }
                            }
                        },
                        Object::InstanceIndex(idx) => {
                            let instance = self.heap.get_instance(idx);
                            // Mark fields hash table
                            roots.extend(instance.fields.values().cloned().collect::<Vec<Value>>());
                            for str_hash in instance.fields.keys() {
                                roots.push(Value::Obj(Object::StringHash(*str_hash)));
                            }
                        },
                        Object::ClassIndex(idx) => {
                            let class = self.heap.get_class(idx);
                            // Mark methods hash table
                            roots.extend(class.methods.values().cloned().collect::<Vec<Value>>());
                            for str_hash in class.methods.keys() {
                                roots.push(Value::Obj(Object::StringHash(*str_hash)));
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
        // Mark hash table
        roots.extend(self.globals.values().cloned().collect::<Vec<Value>>());
        for str_hash in self.globals.keys() {
            roots.push(Value::Obj(Object::StringHash(*str_hash)))
        }
        for callframe in &self.callstack {
            roots.push(Value::Obj(Object::ClosureIndex(callframe.closure_idx)));
        }
        roots.push(Value::object(Object::StringHash(self.init_string_hash)));
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
    #[inline(always)]
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
        return self.stack.get(self.stack_top-1-pos).unwrap();
    }

    /// Method to call a callable object. eg function, native function, instance method, etc..
    fn call_value(&mut self,
                  callee: Value,
                  arg_count: usize)->bool {
        if callee.is_bound_method_index() {
            let bound_idx = callee.as_bound_method_index();
            self.stack[self.stack_top - arg_count - 1] = self.heap.get_bound_method(bound_idx).receiver;
            let closure_idx = self.heap.get_bound_method(bound_idx).closure_idx;
            return self.call(closure_idx, arg_count);
        }
        else if callee.is_closure_index() {
            let closure_idx = callee.as_closure_index();
            return self.call(closure_idx, arg_count);
        } else if callee.is_class_index() {
            let class_idx = callee.as_class_index();
            let instance_idx = self.heap.alloc_instance(Instance::new(class_idx));
            let stack_idx = self.stack_top as isize - (arg_count as isize) - 1;
            self.stack[stack_idx as usize] = Value::Obj(Object::InstanceIndex(instance_idx));

            if self.heap.get_class(class_idx).methods.contains_key(&self.init_string_hash) {
                let initializer = self.heap.get_mut_class(class_idx).methods.get(&self.init_string_hash).unwrap().clone();
                return self.call(initializer.as_closure_index(),arg_count);
            } else if (arg_count != 0) {
                let format = format!("Expect 0 arguments but got {}", arg_count);
                self.runtime_error(&format);
                return false;
            }

            return true;
        } else if callee.is_nativefn_index() {
            let native_fn_idx = callee.as_nativefn_index();
            return self.call_native(arg_count, native_fn_idx);
        }

        self.runtime_error("Can only call function and classes.");
        return false;
    }

    ///
    fn call_native(&mut self, arg_count: usize, native_fn_idx: usize) ->bool {
        let mut native_values: Vec<NativeValue> = vec![];
        self.convert_args_to_native(arg_count, &mut native_values);
        self.fpop(); // pop function
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
    fn convert_args_to_native(&mut self, arg_count: usize, native_values: &mut Vec<NativeValue>) {
        for _ in 0..arg_count {
            let value = self.pop();
            match value {
                Value::Number(n) => native_values.push(NativeValue::Number(n)),
                Value::Bool(b) => native_values.push(NativeValue::Boolean(b)),
                Value::Nil() => native_values.push(NativeValue::Nil()),
                Value::Obj(obj) => match obj {
                        Object::StringHash(hash) => {
                            let str = self.heap.get_string(hash).to_string();
                            native_values.insert(0, NativeValue::String(str));
                        }
                        _ => { panic!("Function, NativeFn are not allowed as argument to native function") }
                }

            }
        }
    }

    /// Insert the call into the call stack
    #[inline(always)]
    fn call(&mut self,
            closure_idx: usize,
            arg_count: usize) -> bool {

        let arity = unsafe { // faster to use ptr
            (*self.heap.functions[(*self.heap.closures[closure_idx].as_ptr()).func_idx].as_ptr()).arity
        };

        // slower => let arity = self.heap.get_function(self.heap.get_closure(closure_idx).func_idx).arity;

        if arg_count != arity {
            let message = format!("Expected {} arguments but got {}", arity, arg_count);
            self.runtime_error(&message);
        }

        let frame = CallFrame::new(closure_idx,
                                   self.stack_top - 1 - arg_count);
        self.callstack.push(frame);
        return true;
    }

    fn define_native(&mut self, name: &str, native: NativeFn) {
        let string_hash = self.heap.alloc_string(name.to_string());
        let native_fn_idx = self.heap.alloc_nativefn(native);
        self.globals.insert(string_hash, Value::Obj(Object::NativeFnIndex(native_fn_idx)));
    }

    /// Reset the stack
    pub fn reset_stack(&mut self) {
        self.stack.clear();
        self.stack_top = 0;
        self.ip = 0;
        self.open_upvalues = None;
        self.curr_func_idx = 0;
        self.callstack.clear();
        self.heap.clear();
    }

    /// Convenience method for binary operations
    fn bin_ops<F>(&mut self, mut apply: F) -> bool
        where F: FnMut(f64, f64)->f64 {

        let b = self.pop();
        let a = self.pop();

        return if Self::is_both_number(&a, &b) {
            self.push(Value::number(apply(a.as_number(), b.as_number())));
            true
        } else {
            self.runtime_error("Operands must be numbers");
            false
        }
    }

    /// Convenience method for binary comparisons
    fn bin_cmp<F>(&mut self, mut apply: F) -> bool
        where F: FnMut(Value,Value)->bool {

        let b = self.pop();
        let a = self.pop();

        return if Self::is_both_number(&a, &b) {
            self.push(Value::bool(apply(a, b)));
            true
        } else {
            self.runtime_error("Operands must be numbers");
            false
        }
    }

    fn close_upvalues(&mut self, frame_slot: usize) {
        while self.open_upvalues_location_greater_or_equal_to(&frame_slot) {
            let location = self.get_open_upvalues_location();
            let value = self.stack.get(location).unwrap().clone();
            self.close_upvalue(value);
            let next = if Self::has_next_upvalue(&mut self.open_upvalues) {
                Self::get_next_upvalue(&self.open_upvalues)
            } else {
                None
            };
            self.open_upvalues = next;
        }
    }

    fn close_upvalue(&mut self, value: Value) {
        self.open_upvalues
            .as_ref().unwrap()  // unwrap option without dropping content
            .as_ref()           // reference the content inside RC
            .borrow_mut().closed = Some(value);
        self.open_upvalues
            .as_ref().unwrap()  // unwrap option without dropping content
            .as_ref()           // reference the content inside RC
            .borrow_mut().location = None;
    }

    fn open_upvalues_location_greater_or_equal_to(&mut self, frame_slot: &usize) -> bool {
        match self.open_upvalues.borrow() {
            Some(it) => it.as_ref().borrow().location.as_ref().unwrap() >= &frame_slot,
            None => false
        }
    }

    fn get_open_upvalues_location(&mut self) -> usize {
        self.open_upvalues
            .as_ref().unwrap()   // unwrap option without dropping
            .as_ref()            // reference the content inside RC
            .borrow()            // borrow upvalue object
            .location.unwrap()   // unwrap option of location
    }

    fn define_method(&mut self, string_hash: u32) {
        let method = self.peek(0);
        let class_idx = self.peek(1).as_class_index();
        self.heap.get_mut_class(class_idx).methods.insert(string_hash, *method);
        self.pop();
    }

    fn bind_method(&mut self, class_idx: usize, name_hash: u32) -> bool {
        if !self.heap.get_class(class_idx).methods.contains_key(&name_hash) {
            let name = self.heap.get_string(name_hash);
            let error_text = format!("Undefined property {}", name);
            self.runtime_error(&error_text);
            return false
        }
        let closure_idx = self.heap.get_class(class_idx).methods.get(&name_hash).unwrap().as_closure_index();
        let bound_method_idx = self.heap.alloc_bound_method(
            BoundMethod::new(*self.peek(0), closure_idx));
        self.pop();
        self.push(Value::Obj(Object::BoundMethodIndex(bound_method_idx)));
        return true;
    }
}
