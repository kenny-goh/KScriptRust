pub struct Closure {
    pub func_idx: usize
}

impl Closure {
    pub fn new(func_idx: usize) -> Closure {
        Closure {
            func_idx
        }
    }
}

pub struct Upvalue {
    pub index: u8,
    pub is_local: bool,
}