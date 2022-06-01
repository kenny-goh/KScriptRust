use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;

pub fn hash_string(t: &String) -> u32 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish() as u32
}

pub fn read_line() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}