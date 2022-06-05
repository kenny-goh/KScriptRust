use std::fmt;
use crate::Object::{ClassIndex, ClosureIndex, FunctionIndex, InstanceIndex, NativeFnIndex};
use crate::object::Object::StringHash;

#[derive(Copy, Clone, Debug)]
pub enum Object {
    StringHash(u32),                // StringHash is a pseudo 'pointer' to the string in the heap via the hash key
    FunctionIndex(usize),           // Function index is a pseudo 'pointer to a function in the heap via index number
    NativeFnIndex(usize),           // Native function index is pseudo 'pointer' to a native function in the heap via index number
    ClosureIndex(usize),            // Closure index is a pseudo 'pointer' to a closure object in the heap via  index number
    ClassIndex(usize),              // Class index is a pseudo pointer to the class object in the heap via index number.
    InstanceIndex(usize),           // Class instance index is a pseudo pointer to the class instance object in the heap via index number.
}

impl Object {
    pub fn string(hash: u32) ->Self {
         StringHash(hash)
    }
    pub fn function(idx: usize) -> Self {
        FunctionIndex(idx)
    }
    pub fn native_fn(idx: usize) -> Self { NativeFnIndex(idx) }
    pub fn closure(idx: usize) -> Self {ClosureIndex(idx) }
    pub fn Class(idx: usize) -> Self { ClassIndex(idx) }
    pub fn Instance(idx: usize) -> Self { InstanceIndex(idx) }

    pub fn as_string_hash(&self) ->u32 {
        return *if let StringHash(ob) = self { ob } else {
            panic!("Not a string")
        };
    }

    pub fn as_function_index(&self) ->usize {
        return *if let FunctionIndex(ob) = self { ob } else {
            panic!("Not a function")
        };
    }

    pub fn as_nativefn_index(&self) ->usize {
        return *if let NativeFnIndex(ob) = self { ob } else {
            panic!("Not a native function")
        };
    }

    pub fn as_closure_index(&self) ->usize {
        return *if let ClosureIndex(ob) = self { ob } else {
            panic!("Not a closure")
        };
    }

    pub fn as_class_index(&self) ->usize {
        return *if let ClassIndex(ob) = self { ob } else {
            panic!("Not a class")
        };
    }

    pub fn as_instance_index(&self) ->usize {
        return *if let InstanceIndex(ob) = self { ob } else {
            panic!("Not an instance")
        };
    }

    pub fn is_string_hash(&self) ->bool {
        return match self {
            StringHash(_) => { true }
            _ => { false }
        }
    }

    pub fn is_function_index(&self) ->bool {
        return match self {
            FunctionIndex(_) => { true }
            _ => { false }
        }
    }
    pub fn is_nativefn_index(&self) ->bool {
        return match self {
            NativeFnIndex(_) => { true }
            _ => { false }
        }
    }

    pub fn is_closure_index(&self) -> bool {
        return match self {
            ClosureIndex(_) => { true }
            _ => false
        }
    }

    pub fn is_class_index(&self) -> bool {
        return match self {
            ClassIndex(_) => { true }
            _ => false
        }
    }

    pub fn is_instance_index(&self) -> bool {
        return match self {
            InstanceIndex(_) => { true }
            _ => false
        }
    }
    
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (StringHash(a), StringHash(b)) => a == b,
            (FunctionIndex(a), FunctionIndex(b)) => a == b,
            (ClosureIndex(a), ClosureIndex(b)) => a == b,
            (ClassIndex(a), ClassIndex(b)) => a == b,
            (InstanceIndex(a), InstanceIndex(b)) => a == b,
            _ => false
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            StringHash(hash) => {
                write!(f, "String hash {}", hash)
            }
            FunctionIndex(idx) => {
                write!(f, "Function index {}", idx)
            }
            NativeFnIndex(idx) => {
                write!(f, "Native fn index {}", idx)
            }
            ClosureIndex(idx) => {
                write!(f, "Closure index {}", idx)
            }
            ClassIndex(idx) => {
                write!(f, "Class index {}", idx)
            }
            InstanceIndex(idx) => {
                write!(f, "Instance index {}", idx)
            }
        }
    }
}

// impl Drop for Object {
//     fn drop(&mut self) {
//         println!("Dropping {}", self)
//     }
// }