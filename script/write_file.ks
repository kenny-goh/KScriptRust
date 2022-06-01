var i = 0;
while( i < 10) {
 appendFile("test.txt", "booyah");
 i = i + 1;
}

for (var i = 0; i < 10; i = i + 1) {
  appendFile("test.txt", "foo bar");
}

