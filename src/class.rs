use fnv::FnvHashMap;
use crate::Value;

pub struct Class {
    pub name: String,
    pub methods: FnvHashMap<u32, Value>
}

impl Class {
    pub fn new(name: String) ->Self {
        Class {
            name,
            methods: Default::default()
        }
    }
}

pub struct Instance {
    pub class_idx: usize,
    pub fields: FnvHashMap<u32, Value>,
}

impl Instance {
    pub fn new(class_idx: usize) ->Self {
        Instance {
            class_idx,
            fields: FnvHashMap::default()
        }
    }
}

pub struct BoundMethod {
    pub receiver: Value,
    pub closure_idx: usize
}


impl BoundMethod {
    pub fn new(receiver: Value, closure_idx: usize) ->Self {
        BoundMethod {
            receiver,
            closure_idx
        }
    }
}