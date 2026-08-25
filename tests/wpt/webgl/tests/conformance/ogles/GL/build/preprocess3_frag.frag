
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
// simple macro expansions.
// Tests for Too few macro arguments, too many macro arguments.
// Macros with no arguments.

#define t1 -1
#define t2 2

#define test -258
#define test1 (test*test)
#define test2(x) (x+test1)
#define test3() (test2(8)*(test*test1))
#define test4(x,y) (x+y)

void main(void)
{
 int sum =0;
 sum = test3();
 sum = test3(3);

 sum = test2(9);
 sum = test2(9,8);

 sum = test4;
 sum = test2(8,5,78,9);
 sum = sum + test1;
 sum = 8+58+sum;
 sum = sum +test;
 sum = (t1+t2);
 sum = test4(test3(),test2(test3()));
 sum = test4(3,8,5);
 sum = test4();
}
