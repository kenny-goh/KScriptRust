use crate::Chunk;

pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
}

impl Function {
    pub fn new(name: String, arity: usize) ->Self {
      Function {
          name,
          arity,
          chunk: Chunk::new()
      }
    }
}