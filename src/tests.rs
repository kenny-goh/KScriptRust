use serial_test::serial;
use std::{fs, mem, thread, time};
use std::fmt::Error;
use crate::{Heap, Parser, RunResult, Scanner, VM};
use crate::nativefn::{clock_native, NativeFn, NativeValue};

/////////////////////////////////////////////////////////////////////
// Tests
/////////////////////////////////////////////////////////////////////

#[test]
#[serial]
fn test_clock_native() {
    let time1 = clock_native(0, vec![]);
    let clock: NativeFn = clock_native;
    thread::sleep(time::Duration::from_millis(1000));
    let time2 = clock(0, vec![]);
    let time1 = match time1 {
        NativeValue::Number(n) => n,
        _=> {panic!("Expected a number.")}
    };
    let time2 = match time2 {
        NativeValue::Number(n) => n,
        _=> {panic!("Expected a number.")}
    };
    assert!(time2-time1 > 0.0);
}

#[test]
#[serial]
fn test_truthy() {
    let code = "true".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_falsy() {
    let code = "false".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("false", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_nil() {
    let code = "nil".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("nil", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_equality_number() {
    let code = "10 == 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_equality_string_truthy() {
    let code = "\"foo\" == \"foo\"".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_equality_string_falsy() {
    let code = "\"foo\" != \"bar\"".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_comparison_greater() {
    let code = "10 > 9".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_comparison_greater_equal() {
    let code = "1 >= 1".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_comparison_lesser() {
    let code = "1 < 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_comparison_lesser_equal() {
    let code = "10 <= 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_comparison_unary_minus() {
    let code = "-100".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("-100", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_comparison_unary_not() {
    let code = "!false".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("true", str),
        Err(_) => panic!("Failed")
    }
}


#[test]
#[serial]
fn test_add() {
    let code = "10 + 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("20", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_subs() {
    let code = "100 - 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("90", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_mul() {
    let code = "100 * 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("1000", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_div() {
    let code = "100 / 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("10", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_multi_binary_ops() {
    let code = "10 + 10 + 20 * 10".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("220", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_global_variable() {
    let code = r#"
        var text = "foo";
        var _result = text;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("foo", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_local_variable() {
    let code = r#"
        var _result = "";
        {
          var text = "foo";
          _result = text;
        }
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("foo", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_nested_local_variable() {
    let code = r#"
        var _result = "";
        {
          {
            var text = "foo";
            _result = text;
          }
        }
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("foo", str),
        Err(_) => panic!("Failed")
    }
}


#[test]
#[serial]
fn test_plus_equal_operator() {
    let code = r#"
        var i = 10;
        i += 100;
        var _result = i;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("110", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_plus_equal_operator_in_for_loop() {
    let code = r#"
        var j = 0;
        for (var i = 0; i <= 100; i+=1) {
            j = i;
        }
        var _result = j;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("100", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_minus_equal_operator() {
    let code = r#"
        var i = 100;
        i -= 10;
        var _result = i;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("90", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_minus_equal_operator_in_for_loop() {
    let code = r#"
        var j = 0;
        for (var i = 100; i >= 0; i-=1) {
            j = i;
        }
        var _result = j;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("0", str),
        Err(_) => panic!("Failed")
    }
}


#[test]
#[serial]
fn test_for_loop() {
    let code = r#"
        var sum = 0;
        for (var i = 0; i < 100; i = i + 1) {
          sum = sum + 1;
        }
        var _result = sum;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("100", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_while_loop() {
    let code = r#"
        var sum = 0;
        while( sum < 100) {
          sum = sum + 1;
        }
        var _result = sum;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("100", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_function_simple() {
    let code = r#"
        fun number() {
          return 1000;
        }
        var _result = number();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("1000", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_function_local_var() {
    let code = r#"
        fun number() {
          var inner = 1000;
          return inner;
        }
        var _result = number();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("1000", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_function_inner_loop() {
    let code = r#"
        fun number() {
          var inner = 0;
          for (var i = 0; i < 100; i = i + 1) {
            inner = inner + 1;
          }
          return inner;
        }
        var _result = number();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("100", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_function_recursive() {
    let code = r#"
       fun fib(n) {
         if (n <= 1) return n;
          return fib(n - 2) + fib(n - 1);
       }
       var _result = fib(19);
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("4181", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_var_shadow_in_scope() {
    let code = r#"
       var x = 100;
       fun shadowVar() {
         var x = 200;
         return x;
       }
       var _result = shadowVar();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("200", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_var_shadow_out_of_scope() {
    let code = r#"
       var x = 100;
       fun shadowVar() {
         var x = 200;
         return x;
       }
       shadowVar();
       var _result = x;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("100", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_stringify() {
    let code = "\"This is a test: \" + str(100.51)".to_string();
    let output = run_expr(&code);
    match output {
        Ok(str) => assert_eq!("This is a test: 100.51", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_closure() {
    let code = r#"
        fun outer() {
          var x = "outside";
          fun inner() {
            return x;
          }
          return inner();
        }
        var _result = outer();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("outside", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_closure_closed_upvalues() {
    let code = r#"
        fun outer() {
          var x = "outside";
          fun inner() {
            return x;
          }
          return inner;
        }
        var closure = outer();
        var _result = closure();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("outside", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_closure_complex_1() {
    let code = r#"
        fun outer() {
          var a = 1;
          var b = 2;

          fun middle() {
            var c = 3;
            var d = 4;
            fun inner() {
              return a + c + b + d;
            }
            return inner;
          }
          return middle;
        }
        var mid = outer();
        var in = mid();
        var _result = in();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("10", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_closure_complex_2() {
    let code = r#"
        var _result = "";
        {
            fun inner() {
                var x = 10;
                fun inner_1() {
                    var y = 20;
                    fun inner_2() {
                        var z = 30;
                        fun inner_3() {
                            _result = x + y + z;
                        }
                        inner_3();
                    }
                    inner_2();
                }
                inner_1();
                }
           inner();
        }
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("60", str),
        Err(_) => panic!("Failed")
    }
}


#[test]
#[serial]
fn test_class_instance_properties() {
    let code = r#"
        class Dude {}
        var dude = Dude();
        dude.name = "Foo man";
        dude.age = 21;
        var _result = dude.name + " " + str(dude.age);
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("Foo man 21", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_class_instance_method() {
    let code = r#"
        class Foo {
          bar() {
            return "bar";
          }
        }
        var f = Foo();
        var _result = f.bar();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("bar", str),
        Err(_) => panic!("Failed")
    }
}

#[test]
#[serial]
fn test_class_instance_method_with_parameter() {
    let code = r#"
        class Foo {
          hi(person) {
            return "Hi " + person;
          }
        }
        var f = Foo();
        var _result = f.hi("Wayne");
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("Hi Wayne", str),
        Err(_) => panic!("Failed")
    }
}


#[test]
#[serial]
fn test_class_instance_this() {
    let code = r#"
        class Foo {
          getName() {
            return this.name;
          }
        }
        var f = Foo();
        f.name = "Foo";
        var _result = f.getName();
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("Foo", str),
        Err(_) => panic!("Failed")
    }
}


#[test]
#[serial]
#[should_panic]
fn test_class_instance_this_outside_method() {
    let code = r#"
        this; // this is illegal
    "#.to_string();
    run_code(&code);
}

#[test]
#[serial]
fn test_class_instance_init() {
    let code = r#"
        class Foo {
          init() {
            this.name = "Foo";
          }
        }
        var f = Foo();
        var _result = f.name;
    "#.to_string();
    let output = run_code(&code);
    match output {
        Ok(str) => assert_eq!("Foo", str),
        Err(_) => panic!("Failed")
    }
}


// todo: garbage collection tests

/////////////////////////////////////////////////////////////////////
// Helper functions
/////////////////////////////////////////////////////////////////////

/// Helper for testing single expression
fn run_expr(code: &String) ->Result<String, Error> {
    let wrapped_code = format!("writeFile(\"result.txt\", str({}));", code);
    // println!("{}", wrapped_code);
    return execute(&wrapped_code);
}

/// Helper for testing multiline code
fn run_code(code: &String) ->Result<String, Error> {
    let wrapped_code = format!("{}\nwriteFile(\"result.txt\", str(_result));", code);
    // println!("{}", wrapped_code);
    return execute(&wrapped_code);
}

/// Interpret and execute the code
fn execute(code: &String) ->Result<String, Error>  {
    let mut vm = VM::new();
    vm.init();

    // Scanning step
    let mut scanner = Scanner::new(&code);
    let tokens = scanner.scan_tokens();

    // transfer heap ownership to parser
    let mut heap_to_parser = Heap::new();
    mem::swap(&mut vm.heap, &mut heap_to_parser);

    // Parsing step
    let mut parser = Parser::new(heap_to_parser, tokens);
    parser.compile();  // pseudo pointer

    // transfer heap ownership of back to vm
    mem::swap(&mut parser.heap, &mut vm.heap, );

    if parser.had_error {
        panic!("Parsing failed with error.");
    }

    // Execution step
    let result = vm.execute();
    match result {
        RunResult::Ok => {
            let contents = fs::read_to_string("result.txt")
                .expect("Something went wrong reading the file");
            return Ok(contents.trim().to_string());
        }
        _ => {
            panic!("VM failed to execute.");
        }
    }
}