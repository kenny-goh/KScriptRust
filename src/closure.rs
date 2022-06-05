use std::cell::RefCell;
use std::rc::Rc;
use crate::{Value, VM};

pub struct Closure {
    pub func_idx: usize,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>
}

impl Closure {
    pub fn new(func_idx: usize) -> Closure {
        Closure {
            func_idx,
            upvalues: vec![]
        }
    }
    pub fn init_upvalues(&mut self, upvalue_count: usize) {
        for _ in 0..upvalue_count {
            self.upvalues.push(Rc::new(RefCell::new(ObjUpvalue::as_null())));
        }
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.func_idx == other.func_idx
    }
}

pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

impl Upvalue {
    pub fn new(index: usize, is_local: bool) ->Upvalue {
        Upvalue {
            index,
            is_local
        }
    }
}

impl Clone for Upvalue {
    fn clone(&self) -> Self {
        return Upvalue::new(self.index, self.is_local);
    }
}

pub struct ObjUpvalue {
    pub is_null: bool,
    pub location: Option<usize>,
    pub closed: Option<Value>,
    pub next: Option<Rc<RefCell<ObjUpvalue>>>
}

impl ObjUpvalue {
    pub fn as_null() -> ObjUpvalue {
        ObjUpvalue {
            is_null: true,
            location: None,
            closed: None,
            next: None,
        }
    }

    pub fn new(location: usize,
               next: Option<Rc<RefCell<ObjUpvalue>>> ) -> ObjUpvalue {
        ObjUpvalue {
            is_null: false,
            location: Some(location),
            closed: None,
            next
        }
    }


    pub fn resolve_value(&mut self, vm: &VM) -> Value {
        return if self.closed.is_some() {
            self.closed.unwrap()
        } else if self.location.is_some() {
            vm.stack[self.location.unwrap()]
        } else {
            panic!("Unreachable code");
        }
    }
}
