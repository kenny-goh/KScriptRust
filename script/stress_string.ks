var str = "a";
for (var i = 0; i < 100000; i = i + 1) {
  str = str + "a";
}
writeFile("result.txt", str);
