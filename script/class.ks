class Dude {
    init() {
      this.name = "Foo man";
    }
    greet(name) {
       print "Hi " + name + "!";
    }
    printName() {
      print "Name is: " + this.name;
    }
}

var dude = Dude();
print "Name: " + dude.name;
dude.greet("Alex");
dude.printName();

