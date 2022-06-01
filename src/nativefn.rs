use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(u8, Vec<NativeValue>) -> NativeValue;

pub enum NativeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil(),
}

// fixme: Replace NativeValue with Result<NativeValue,Error>

///
pub fn str_native(arg_count: u8, arguments: Vec<NativeValue>) -> NativeValue {
    return match arguments.get(0).unwrap() {
        NativeValue::String(s) => NativeValue::String(s.to_string()),
        NativeValue::Number(n) => NativeValue::String(n.to_string()),
        NativeValue::Boolean(b) => NativeValue::String(b.to_string()),
        NativeValue::Nil() => NativeValue::String("nil".to_string())
    };
}

///
pub fn clock_native(arg_count: u8, arguments: Vec<NativeValue>) -> NativeValue {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH);
    return NativeValue::Number(since_the_epoch.unwrap().as_secs_f64())
}

///
pub fn write_file_native(arg_count: u8, arguments: Vec<NativeValue>) -> NativeValue {

    //fixme: check vec is equal to arg count

    let path = match arguments.get(0).unwrap() {
        NativeValue::String(str) => { str }
        _ => { panic!("Invalid type for path, string expected.") } // replace with error
    };

    let content = match arguments.get(1).unwrap() {
        NativeValue::String(str) => { str }
        _ => {
            panic!("Invalid type for content, string expected"); // replace with error
        }
    };

    write_file(path, content);

    return NativeValue::Boolean(true);
}

pub fn append_file_native(arg_count: u8, arguments: Vec<NativeValue>) -> NativeValue {

    //fixme: check vec is equal to arg count

    let path = match arguments.get(0).unwrap() {
        NativeValue::String(str) => { str }
        _ => { panic!("Invalid type for path, string expected.") } // replace with error
    };

    let content = match arguments.get(1).unwrap() {
        NativeValue::String(str) => { str }
        _ => {
            panic!("Invalid type for content, string expected"); // replace with error
        }
    };

    append_file(path, content);

    return NativeValue::Boolean(true);
}

fn write_file(path: &str, content: &str) {
    let mut f = File::create(path).unwrap();
    let lines = content.split("\\n");
    for line in lines {
        writeln!(&mut f, "{}", line).unwrap();
    }
}

fn append_file(path: &str, content: &str) {
    let mut f = OpenOptions::new().write(true).create(true).append(true).open(path).unwrap();
    let lines = content.split("\\n");
    for line in lines {
        writeln!(&mut f, "{}", line).unwrap();
    }
}