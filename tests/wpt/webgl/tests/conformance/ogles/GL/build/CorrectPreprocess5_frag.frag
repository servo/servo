
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
//mutiple line macros - test case.

#define test 5
#define t1 1
#define t2 2
#define token (t1+t2)
#define test1 int sum =1; sum = test; sum = test+test;

#define test2 { test1 sum = sum +token; sum = t2*t1; }

void main(void)
{
 int test3=1;
 test1
 test2;
 test3 = test;
 sum = test3;
}


