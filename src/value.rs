use std::cmp::Ordering;
use std::fmt;
use crate::object::{Object};
use crate::Value::{Bool, Nil, Number, Obj};

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Obj(Object),
    Nil(),
}

impl Value {
    pub fn nil() ->Self {
        Nil()
    }
    pub fn bool(boolean: bool) -> Self {
       Bool(boolean)
    }
    pub fn number(number: f64) -> Self {
       Number(number)
    }
    pub fn object(object: Object) -> Self {
        Obj(object)
    }

    pub fn as_number(&self) ->f64 {
        return *if let Number(r) = self { r } else {
            panic!("Not a number")
        };
    }

    pub fn as_boolean(&self) ->bool {
        return *if let Bool(r) = self { r } else {
            panic!("Not a boolean")
        };
    }

    pub fn as_object(&self) ->&Object {
        return if let Obj(ob) = self { ob } else {
            panic!("Not an object")
        };
    }

    pub fn as_string_hash(&self) ->u32 {
        return if let Obj(ob) = self { ob.as_string_hash() } else {
            panic!("Not an object")
        };
    }

    pub fn as_function_index(&self) ->usize {
        return if let Obj(ob) = self { ob.as_function_index() } else {
            panic!("Not a function object")
        };
    }

    pub fn as_nativefn_index(&self) ->usize {
        return if let Obj(ob) = self { ob.as_nativefn_index() } else {
            panic!("Not a nativefn object")
        };
    }

    pub fn as_closure_index(&self) ->usize {
        return if let Obj(ob) = self { ob.as_closure_index() } else {
            panic!("Not a closure object")
        };
    }

    pub fn as_class_index(&self) ->usize {
        return if let Obj(ob) = self { ob.as_class_index() } else {
            panic!("Not a class object")
        };
    }

    pub fn as_instance_index(&self) ->usize {
        return if let Obj(ob) = self { ob.as_instance_index() } else {
            panic!("Not a class instance")
        };
    }

    pub fn is_number(&self) ->bool {
        return match self {
            Number(_) => { true }
            _ => { false }
        }
    }

    pub fn is_object(&self) ->bool {
        return match self {
            Obj(_) => { true }
            _ => { false }
        }
    }

    pub fn is_boolean(&self) ->bool {
        return match self {
            Bool(_) => { true }
            _ => { false }
        }
    }

    pub fn is_string_hash(&self) -> bool {
        return match self {
            Obj(obj) => {obj.is_string_hash()}
            _ => { false }
        }
    }
    pub fn is_function_index(&self) -> bool {
        return match self {
            Obj(obj) => {obj.is_function_index()}
            _ => { false }
        }
    }
    pub fn is_nativefn_index(&self) -> bool {
        return match self {
            Obj(obj) => {obj.is_nativefn_index()}
            _ => { false }
        }
    }
    pub fn is_closure_index(&self) -> bool {
        return match self {
            Obj(obj) => {obj.is_closure_index()}
            _ => { false }
        }
    }

    pub fn is_class_index(&self) -> bool {
        return match self {
            Obj(obj) => {obj.is_class_index()}
            _ => { false }
        }
    }

    pub fn is_instance_index(&self) -> bool {
        return match self {
            Obj(obj) => {obj.is_instance_index()}
            _ => { false }
        }
    }

}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Nil(), Nil()) => true,
            (Obj(a), Obj(b)) => a == b,
            _ => false
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self, &other) {
            (Number(a), Number(b)) => a.partial_cmp(b),
            _ => {
                panic!("Unreachable code")
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Number(val) => {
                write!(f, "{}", val)
            }
            Bool(boolean) => {
                write!(f, "{}", boolean)
            }
            Nil() => {
                write!(f, "nil")
            }
            Obj(object) => {
                write!(f, "{}", &object)
            }
        }
    }
}

// impl Drop for Value {
//     fn drop(&mut self) {
//         println!("Dropping {}", self)
//     }
// }


