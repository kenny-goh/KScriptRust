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
var result = in();
print result;