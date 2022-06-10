class Parent {
    init() {
      this.parent_field = "i am parent";
    }
    method() {
        print "parent's method";
    }
    overrideMethod() {
       print "parent's overrideMethod";
    }
}

class Child extend Parent {
    overrideMethod() {
      super.overrideMethod();
      print "Child's override method";
      print this.parent_field;
    }
}

var c = Child();
c.method();
c.overrideMethod();

