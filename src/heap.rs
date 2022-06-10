use std::borrow::{Borrow};
use std::cell::{Ref, RefCell, RefMut};
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::mem;

use colored::Colorize;

use crate::{Value};
use crate::class::{Class, Instance};
use crate::function::Function;
use crate::nativefn::NativeFn;
use crate::closure::Closure;
use crate::utils::hash_string;

const GC_FACTOR: usize = 2;
const INITIAL_SIZE: usize = 1024 * 1024;

/// Heap is an object responsible for managing the lifecycle of all the
/// resources that needs to be stored on the heap. The resources are
/// owned by the heap.
///
///
/// Objects that need to access the resources for read operations will
/// need to use hash key as a pseudo pointer.
pub struct Heap {
    /// Current size consumed in terms of bytes for heap memory
    pub bytes_allocated: usize,
    /// Next gc point in terms of memory size in bytes
    pub next_gc: usize,
    /// Storage for strings.
    pub strings: HashMap<u32, Box<String>>,
    /// Storage for functions. Function is mutable, hence the use of RefCell
    pub functions: Vec<RefCell<Function>>,
    /// Storage for native functions
    pub native_fns: Vec<Box<NativeFn>>,
    /// Storage for closures
    pub closures: Vec<RefCell<Closure>>,
    /// Storage for classes
    pub classes: Vec<RefCell<Class>>,
    /// Storage for class instances
    pub instances: Vec<RefCell<Instance>>, // fixme: this should be a hash map with unique identifier for each instance and BOXED too
}


impl Heap {
    pub fn new() ->Self {
        Heap {
            bytes_allocated: 0,
            next_gc: INITIAL_SIZE,
            strings: Default::default(),
            functions: vec![],
            native_fns: vec![],
            closures: vec![],
            classes: vec![],
            instances: vec![],
        }
    }

    /// Allocate string object
    pub fn alloc_string(&mut self, string: String) -> u32 {
        let hash = hash_string(&string);
        let size = mem::size_of_val(&string);
        if !self.strings.contains_key(&hash) {
            self.bytes_allocated += size;
            self.strings.insert(hash, Box::new(string));
        }
        return hash;
    }

    /// Allocate function object
    pub fn alloc_function(&mut self, function: Function) -> usize {
        let size = mem::size_of_val(&function);
        self.bytes_allocated += size;
        let size = self.functions.len();
        self.functions.push(RefCell::new(function));
        return size;
    }

    /// Allocate native fn
    pub fn alloc_nativefn(&mut self, function: NativeFn) -> usize {
        // let hash = hash_string(&function.name);
        let size = mem::size_of_val(&function);
        self.bytes_allocated += size;
        let size = self.native_fns.len();
        self.native_fns.push(Box::new(function));
        return size;
    }

    /// Allocate closure
    pub fn alloc_closure(&mut self, closure: Closure) -> usize {
        let size = mem::size_of_val(&closure);
        self.bytes_allocated += size;
        let size = self.closures.len();
        self.closures.push(RefCell::new(closure));
        return size;
    }

    /// Allocate class
    pub fn alloc_class(&mut self, class: Class) -> usize {
        let size = mem::size_of_val(&class);
        self.bytes_allocated += size;
        let size = self.classes.len();
        self.classes.push(RefCell::new(class));
        return size;
    }

    /// Allocate instance
    pub fn alloc_instance(&mut self, instance: Instance) ->usize {
        let size = mem::size_of_val(&instance);
        self.bytes_allocated += size;
        let size = self.instances.len();
        self.instances.push(RefCell::new(instance));
        return size;
    }

    pub fn is_ready_for_garbage_collection(&self) ->bool {
        return self.bytes_allocated > self.next_gc;
    }

    ///
    pub fn run_gc(&mut self, marked: Vec<Value>) {
        let string_heap_len_before_gc = self.strings.len();
        let closure_heap_len_before_gc = self.closures.len();
        let func_heap_len_before_gc = self.functions.len();
        let before_gc =  self.bytes_allocated as f32 / 1000000.0;

        self.sweep(marked);
        let after_gc = self.bytes_allocated as f32 / 1000000.0;
        self.next_gc = cmp::max(self.bytes_allocated * GC_FACTOR, INITIAL_SIZE);

        let next_gc = self.next_gc as f32 / 1000000.0;
        let string_heap_len_after_gc = self.strings.len();
        let closure_heap_len_after_gc = self.closures.len();
        let func_heap_len_after_gc = self.functions.len();

        println!("{} Freed memory from {:.2} MB to {:.2} MB, next GC at {:.2} MB.", "GC".bold().blue(), before_gc, after_gc, next_gc);
        if string_heap_len_before_gc != string_heap_len_after_gc {
            println!("{} Reduced string capacity from {} to {}", "GC".bold().blue(), string_heap_len_before_gc, string_heap_len_after_gc);
        }
        if closure_heap_len_before_gc != closure_heap_len_after_gc {
            println!("{} Reduced closure capacity from {} to {}", "GC".bold().blue(), closure_heap_len_before_gc, closure_heap_len_after_gc);
        }
        if func_heap_len_before_gc != func_heap_len_after_gc {
            println!("{} Reduced function capacity from {} to {}", "GC".bold().blue(), func_heap_len_before_gc, func_heap_len_after_gc);
        }
    }


    /// Sweep orphan objects from the heap after comparing with the marked values
    fn sweep(&mut self, marked: Vec<Value>) {
        self.free_strings(&marked);
        self.free_closures(&marked);
        self.free_functions(&marked);
        self.free_classes(&marked);
        self.free_instances(&marked);
    }

    fn free_strings(&mut self, marked: &Vec<Value>) {
        let mut is_alive: HashSet<u32> = HashSet::new();
        for each in marked {
            if each.is_string_hash() {
                is_alive.insert(each.as_string_hash());
            }
        }
        let mut deletions: HashSet<u32> = HashSet::new();
        //  deleting strings
        for each in self.strings.keys() {
            if is_alive.contains(each) {
                continue;
            }
            let string = self.strings.get(&each).unwrap();
            let size = mem::size_of_val(&string);
            if self.bytes_allocated > size {
                self.bytes_allocated -= size;
            }
            deletions.insert(*each);
        }
        for each in deletions {
            self.strings.remove(&each);
        }
    }

    fn free_closures(&mut self, marked: &Vec<Value>) {
        let mut is_alive: HashSet<usize> = HashSet::new();
        for each in marked {
            if each.is_closure_index() {
                is_alive.insert(each.as_closure_index());
            }
        }
        let mut deletions: Vec<usize> = vec![];
        let mut index = 0;
        for each in &self.closures {
            if is_alive.contains(&index) {
                continue;
            }
            let size = mem::size_of_val(&each);
            if self.bytes_allocated > size {
                self.bytes_allocated -= size;
            }
            deletions.push(index);
            index += 1;
        }

        deletions.sort();
        deletions.reverse();

        for i in 0..deletions.len() {
            let index = deletions[i];
            self.closures.remove(index);
        }
    }

    // fixme: this can be a generic function
    fn free_functions(&mut self, marked: &Vec<Value>) {
        let mut is_alive: HashSet<usize> = HashSet::new();
        for each in marked {
            if each.is_function_index() {
                is_alive.insert(each.as_function_index());
            }
        }
        let mut deletions: Vec<usize> = vec![];
        let mut index = 0;
        for each in &self.functions {
            if is_alive.contains(&index) {
                continue;
            }
            let size = mem::size_of_val(&each);
            if self.bytes_allocated > size {
                self.bytes_allocated -= size;
            }
            deletions.push(index);
            index += 1;
        }

        deletions.sort();
        deletions.reverse();

        for i in 0..deletions.len() {
            let index = deletions[i];
            self.functions.remove(index);
        }
    }

    fn free_classes(&self, marked: &Vec<Value>) {
        todo!()
    }

    fn free_instances(&self, marked: &Vec<Value>) {
        todo!()
    }

    /// Access string via hash key
    pub fn get_string(&self, hash: u32) ->&String {
        return self.strings.get(&hash).unwrap();
    }

    /// Mutator access function via index number
    pub fn get_mut_function(&self, idx: usize) -> RefMut<'_, Function> { self.functions[idx].borrow_mut() }

    /// NonMutator access function via index number
    pub fn get_function(&self, idx: usize) -> Ref<'_, Function> { self.functions[idx].borrow() }

    ///
    pub fn get_nativefn(&self, idx: usize)->&NativeFn { self.native_fns[idx].borrow() }

    /// Mutator access closure via index number
    pub fn get_mut_closure(&self, idx: usize) -> RefMut<'_, Closure> { self.closures[idx].borrow_mut() }

    /// Non mutator access closure via index number
    pub fn get_closure(&self, idx: usize) -> Ref<'_, Closure> { self.closures[idx].borrow() }

    /// Mutator access class via index number
    pub fn get_mut_class(&self, idx: usize) -> RefMut<'_, Class> { self.classes[idx].borrow_mut() }

    /// Non mutator access class via index number
    pub fn get_class(&self, idx: usize) -> Ref<'_, Class> { self.classes[idx].borrow() }

    /// Mutator instance class via index number
    pub fn get_mut_instance(&self, idx: usize) -> RefMut<'_, Instance> { self.instances[idx].borrow_mut() }

    /// Non mutator access instance via index number
    pub fn get_instance(&self, idx: usize) -> Ref<'_, Instance> { self.instances[idx].borrow() }

    /// Clear the heap - for testing only
    pub fn clear(&mut self) {
        self.strings.clear();
        self.functions.clear();
        self.classes.clear();
        self.closures.clear();
        self.instances.clear();
        self.bytes_allocated = 0;
        self.next_gc = INITIAL_SIZE;
    }


}

impl Drop for Heap {
    fn drop(&mut self) {
        // println!("DROPPING HEAP");
        // self.strings.clear();
    }
}
