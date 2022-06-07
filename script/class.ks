class Dude {
    greet(name) {
       print "Hi " + name + "!";
    }
}
var dude = Dude();
dude.name = "Foo man";
print dude.name;
dude.greet("Alex");
