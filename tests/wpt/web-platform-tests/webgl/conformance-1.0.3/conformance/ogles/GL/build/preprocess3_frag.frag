
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
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
