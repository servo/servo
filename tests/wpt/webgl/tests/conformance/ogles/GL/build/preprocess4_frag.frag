
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
// #error and #pragma directives -- test cases.
// tests for errors in #pragma directive.

#pragma optimize(on)
#pragma debug(off)

int foo(int);

void main(void)
{
 int sum =0;
 #error ;
 #error 78
 #error c
 #error "message to the user "
 #error message to the user
 #error
 #error
 #define t1 1
 sum = t1*t1;
 foo(sum);

}

#pragma optimize(off)
#pragma bind(on)
#pragma pack(off)

int foo(int test)
{
 int binding=0;
 binding = test;
 return binding;
}

#line 4
#pragma
#line 5 6
#pragma optmimize on
#pragma debug off
#pragma debug(off
#line 9
#prgma bind(off)
#pragma bind
#pragma (on)
#pragma on (on)
#pragma optmize(on


