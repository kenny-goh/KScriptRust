use fnv::FnvHashMap;
use crate::Value;

pub struct ClassInstance {
    pub class_idx: usize,
    pub fields: FnvHashMap<u32, Value>,
    pub is_tombstone: bool,
}

impl ClassInstance {
    pub fn new(class_idx: usize) ->Self {
        ClassInstance {
            class_idx,
            fields: FnvHashMap::default(),
            is_tombstone: false
        }
    }
}