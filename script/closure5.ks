{
  fun inner() {
     var x = 10;
     fun inner_1() {
       var y = 20;
       fun inner_2() {
         var z = 30;
         fun inner_3() {
            print x + y + z;
         }
         inner_3();
       }
       inner_2();
     }
     inner_1();
  }
  inner();
}