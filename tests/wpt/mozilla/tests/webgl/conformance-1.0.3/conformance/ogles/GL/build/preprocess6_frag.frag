
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
// operator precedence and some macro expansions.

#define test (1+2)
#define test1 (test*4)
#define test2 (test1/test)
//#define test3 (-1+2*3/4%test)
#define test3 (-1+2*3/4)
//#define test4 (test & test1 |test2)
#define test4 (test)
#define test5 (!8+~4+4-6)
#define test6 (test1>>1)
#define test7 (test1<<1)
#define test8 (test2^6)
#define test9 (test4 || test5 && test1)
#define test10 (0)

void main(void)
{
 int sum =0;
 sum = test4;
 sum = test3*test2+test1-test;
// sum = test3/test6 + test4*test7 - test7 % test9;
// sum = test3/test6 + test4*test7 - test7;
 sum = test10*test5;
}

