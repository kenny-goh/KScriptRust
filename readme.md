KScriptRust is a scripting language influenced by Javascript and Python. The language is implemented using 
single pass compiler + virtual machine written in Rust.

The implementation details of this language is based on the learnings from the Crafting Interpreters 
book by Bob Nystrom.

Why KScriptRust? 
Firstly, this is an excuse for me to improve my skills in Rust as I am fascinated by Rust.  Secondly, I have always been curious with how virtual machine (such as Java virtual machine or CPython) is implemented, 
this project is an opportunity for me to scratch that itch.

## Installation
Install rust on your machine. Please refer to  https://www.rust-lang.org/

## Usage
```shell
# Build Kscript 
cargo build --release # This will generate kscript binary in target/release

# Run kscript in interactive mode
./target/release/kscript_rust 

# Run kscript with fibonacci script
./target/release/kscript_rust ./script/fib.ks
```

## Example kscript program
```shell

// Print 
print "hello world";  // "hello world"
print 100 + 200;      // "300"

// Expression
20 + 20 + 30 * 2; // Evaluates to 100

// Variable
var foo = "bar";
print foo;        // "bar"

// For loop
for (var i = 0; i < 10; i += 1) {
  // do something
}


// Native functions

// str(object)
var mergeString = "Number is " + str(100) // "Number is 100"

// clock
var t1 = clock();
var t2 = clock();
print t2 - t1;

// Functions
fun foo() {
  print "foo";
}
foo();

// Fibonacci example
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}
for (var i = 0; i < 30; i = i + 1) {
  print fib(i);
}

// Object oriented programming

class LinkedNode {
   init(value, linkedNode) {
     this.value = value;
     this.next = linkedNode;
   }
}

class LinkedList {
  init() {
    this.head = nil;
    this.current = nil;
    this.length = 0;
  }
  add(value) {
    var node = LinkedNode(value, nil);
    if (this.head == nil) {
      this.head = node;
      this.head.next = node;
    }
    else if (this.current != nil) {
        this.current.next = node;
    }
    this.current = node;
    this.length = this.length + 1;
  }
}

```
For more examples, please refer to script subdirectory

### Example byte codes (in disassembled mode)

An example kscript program
```shell
print 10+10+20*50;
for (var i = 0; i < 100; i += 1) { 
  print i; 
}
```

Compiled bytecodes
```shell
main
Loc  | Line  | Instruction          | Const  | Values
   0 |     0 | op_constant          |      0 | 10
   2 |     0 | op_constant          |      0 | 10
   4 |     0 | op_add
   5 |     0 | op_constant          |      1 | 20
   7 |     0 | op_constant          |      2 | 50
   9 |     0 | op_mul
  10 |     0 | op_add
  11 |     0 | op_print
  12 |     1 | op_constant          |      3 | 0
  14 |     1 | op_get_local         |      1 |
  16 |     1 | op_constant          |      4 | 100
  18 |     1 | op_less
  19 |     1 | op_jump_if_false     | 19 => 43
  22 |     1 | op_pop
  23 |     1 | op_jump              | 23 => 37
  26 |     1 | op_get_local         |      1 |
  28 |     1 | op_constant          |      5 | 1
  30 |     1 | op_add
  31 |     1 | op_set_local         |      1 |
  33 |     1 | op_pop
  34 |     1 | op_loop              | 34 => 14
  37 |     2 | op_get_local         |      1 |
  39 |     2 | op_print
  40 |     3 | op_loop              | 40 => 26
  43 |     3 | op_pop
  44 |     3 | op_pop
  45 |     3 | op_nil
  46 |     3 | op_return
```

## Grammar

Statement
* program -> declaration * EOF
* statement -> expr_stmt | print_stmt | for | if | while | return
* expr_stmt ->  expression “;”
* print_stmt -> “print” expression “;”
* block -> “{“ declaration * “}”
* declaration -> fun_decl | var_decl | statement;
* var_decl -> "var" IDENTIFIER;
* fun_decl -> "fun" function;
* function -> IDENTIFIER "(" parameters? ")" block;
* parameters-> IDENTIFIER "(," IDENTIFIER ")"*;

Expression
* expression -> equality;
* equality -> comparison ( (“!=”|”==”) comparison )*;
* comparison -> term( (“>”|”>=”|”<”|”<=) term)*;
* term-> factor( (“-”|”+”) factor)*;
* factor-> unary( (“/”|”*”) unary)*;
* literal -> number | string | “true” | “false” | “nil”;
* unary -> (“-” | “!”) unary | primary;
* binary -> expression operator expression;
* operator -> “==” | “!=” | “<” | “<=” | “>” | “>=” | “+” | “-” | “*” | “/”;
* primary -> number | string | “true” | “false” | “nil” | “(“ expression “)”;
* arguments -> expression ( "," expression )*
* call-> primary "("  arguments? ")" block


## Performance 

Using the following inefficient method of computing fibonaci sum

KScript
```shell
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}
print fib(40);
```

Python
```shell
def fib(n):
  if n <= 1:
    return n;
  return fib(n - 2) + fib(n - 1)
fib(40)
```

The benchmark is:  
KScript ~ 19s  (not too shabby without deep finetuning)  
Python ~ 17s

I will optimize KScriptRust once I have fully implemented object-oriented features and completed the garbage collector.

## Todos
- GC (Partially working, will need to add for classes)
- let operator (immutable variable)
- lambda function
- Array types 
- Hashmap types
- Outputting compiled machine code as [filename].bin
- Non-blocking IO (using Tokio crate) 
- Sockets
- Runtime statistics / profiling 
- Parallel KScript (async) - Need research

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)# kscript
