KScriptRust is a single pass interpreter + virtual machine written in Rust.

Why? Opportunity for me to improve my skills in Rust and also learn about writing a virtual machine.

## Installation
Install rust on your machine. Please refer to  https://www.rust-lang.org/

## Usage
```shell
# Run example fibonacci script
cargo run --package kscript_rust --bin kscript_rust ./script/fib.ks

# Run interactive mode
cargo run --package kscript_rust --bin kscript_rust 
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
for (var i = 0; i < 10; i = i + 1) {
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

```
For more examples, please refer to script subdirectory

## Todos
- Closure (Currently work in progress)
- Classes
- GC (Partially working, will need to add for classes)
- let operator (immutable variable)
- += operator
- ++ operator
- lambda function
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
