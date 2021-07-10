
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


