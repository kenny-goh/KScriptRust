
#[derive(Copy, Clone, Eq, PartialEq)]

/// Represent runtime call data structure
pub struct CallFrame {
    /// Pseudo pointer to the closure object in the heap
    pub closure_idx: usize,
    /// Allows VM to store the last known instruction pointer index before jumping to another function
    pub ip: usize,
    /// Represent the 'starting' offset for all the variables for this call frame from the VM stack
    pub slot_offset: usize
}
impl CallFrame {
    pub fn new(closure_idx: usize, slot_offset: usize) ->Self {
        CallFrame {
            closure_idx,
            slot_offset,
            ip: 0,
        }
    }
}