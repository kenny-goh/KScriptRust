use crate::{Chunk, Heap, Object, Opcode, Value};


fn simple_instruction(name: &str, offset: usize) ->usize {
    println!("{}", name);
    return offset + 1;
}

fn constant_instruction(name: &str, chunk: &Chunk, heap: &Heap, offset: usize) ->usize {
    let constant = chunk.code.get(offset + 1).unwrap();
    print!("{: <20} | {: >6} | ", name, constant);
    let value = chunk.constants.get(*constant as usize).unwrap();
    match value {
        Value::Obj(object) => {
            match object {
                Object::StringHash(str_hash) => {
                    let str = heap.get_string(*str_hash);
                    println!("{: <20}", str);
                }
                Object::FunctionIndex(idx) => {
                    let fun = heap.get_mut_function(*idx);
                    println!("{: <20}", format!("<fn {}>",fun.name));
                }
                Object::NativeFnIndex(_) => {
                    println!("{: <20}","<nativefn>");
                }
                Object::ClosureIndex(idx) => {
                    let closure = heap.get_mut_closure(*idx);
                    let func_idx = closure.func_idx as usize;
                    let func = heap.get_mut_function(func_idx);
                    println!("{: <20}", format!("<fn {}>", func.name));
                }
            }
        }
        _ => {
            println!("{: <20}", chunk.constants.get(*constant as usize).unwrap());
        }
    }
    return offset + 2;
}

fn  byte_instruction(name: &str, chunk: &Chunk, offset: usize)->usize {
    let slot = chunk.code.get(offset + 1).unwrap();
    println!("{: <20} | {: >6} | ", name, slot);
    return offset + 2;
}

#[allow(arithmetic_overflow)]
fn  jump_instruction(name: &str, sign: isize, chunk: &Chunk, offset: usize)->usize {
    let mut jump:u32 = (chunk.code[offset + 1] as u32) << 8;
    jump |= (chunk.code[offset+2]) as u32;
    // printf("%-16s | %04d -> %04d\n", name, offset, offset + 3 + sign * jump);
    println!("{: <20} | {} => {}", name, offset, offset as isize + 3 + sign * jump as isize);
    return offset + 3;
}

pub fn disassemble_chunk(chunk: &Chunk, heap: &Heap, name: &str) {
    println!("{}", name);
    println!("Loc  | Line  | instruction          | const  | values");
    let mut offset = 0;
    loop {
        if offset >= chunk.code.len() { break };
        offset = disassemble_instruction(chunk, heap, offset);
    }
}

fn disassemble_instruction(chunk: &Chunk, heap: &Heap, mut offset: usize) -> usize {
    print!("{: >4} | {: >5 } | ", offset, chunk.lines[offset]);
    let inst = chunk.code.get(offset).unwrap().clone();
    let opcode: Opcode = unsafe { std::mem::transmute(inst) };
    match opcode {
        Opcode::Constant => {
            return constant_instruction( "op_constant", chunk, heap, offset);
        }
        Opcode::Nil => {
            return simple_instruction("op_nil", offset);
        }
        Opcode::True => {
            return simple_instruction("op_true", offset);
        }
        Opcode::False => {
            return simple_instruction("op_false", offset);
        }
        Opcode::Pop => {
            return simple_instruction("op_pop", offset);
        }
        Opcode::GetLocal => {
            return byte_instruction("op_get_local", chunk,  offset);
        }
        Opcode::GetGlobal => {
            return constant_instruction("op_get_global", chunk, heap, offset);
        }
        Opcode::DefineGlobal => {
            return constant_instruction("op_define_global", chunk, heap, offset);
        }
        Opcode::SetLocal => {
            return byte_instruction("op_set_local", chunk, offset);
        }
        Opcode::SetGlobal => {
            return constant_instruction("op_get_global", chunk, heap, offset);
        }
        Opcode::GetUpvalue => {
            return byte_instruction("op_get_upvalue", chunk, offset);
        }
        Opcode::SetUpvalue => {
            return byte_instruction("op_set_upvalue", chunk, offset);
        }
        Opcode::Equal => {
            return simple_instruction("op_equal", offset);
        }
        Opcode::Greater => {
            return simple_instruction("op_greater", offset);
        }
        Opcode::Less => {
            return simple_instruction("op_less", offset);
        }
        Opcode::Add => {
            return simple_instruction("op_add", offset);
        }
        Opcode::Subtract => {
            return simple_instruction("op_subtract", offset);
        }
        Opcode::Multiply => {
            return simple_instruction("op_mul", offset);
        }
        Opcode::Divide => {
            return simple_instruction("op_divide", offset);
        }
        Opcode::Not => {
            return simple_instruction("op_not", offset);
        }
        Opcode::Negate => {
            return simple_instruction("op_negate", offset);
        }
        Opcode::Print => {
            return simple_instruction("op_print", offset);
        }
        Opcode::JumpIfFalse => {
            return jump_instruction("op_jump_if_false", 1, chunk, offset);
        }
        Opcode::Jump => {
            return jump_instruction("op_jump", 1, chunk, offset);
        }
        Opcode::Loop => {
            return jump_instruction("op_loop", -1, chunk, offset);
        }
        Opcode::Call => {
            return byte_instruction("op_call", chunk, offset);
        }
        Opcode::Closure => {
            offset += 1;
            let constant = chunk.code[offset] as usize;
            offset += 1;
            let value = chunk.constants[constant];
            print!("{:>4} {:>5 }", "OP_CLOSURE" , constant);
            println!("  {:>10}", value);
            let func_index = value.as_function_index();
            let function = heap.get_mut_function(func_index);
            for _ in 0..function.upvalue_count {
                let is_local = chunk.code[offset];
                offset+=1;
                let index = chunk.code[offset];
                offset+=1;
                let local_str = if is_local == 0u8 {"local"} else {"upvalue"};
                println!("{:>4}           | {:>4}{:>2 }", offset - 2, local_str , index)
            }
            return offset;
        }
        Opcode::CloseValue => {
            return 0;
        }
        Opcode::Return => {
            return simple_instruction("op_return", offset);
        }
    }
}
