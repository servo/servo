
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
vec4 test_function4(float);
vec4 test_function1(float);
vec4 test_function2(float);
vec4 test_function3(float);
float f = 2.6;


vec4 test_function1(float ff)
{
    vec4 func_vec4 = vec4(ff+f);
    return func_vec4;
}

float f1 = 1.5;

vec4 test_function4(float ff)
{
    vec4 func_vec4 = vec4(f1);
    return func_vec4;
}

float f2 = 3.5;

void main()
{
    vec4 v1 = test_function4(f2);
    vec4 v2 = test_function1(f2);
    vec4 v3 = test_function2(f2);
    vec4 v4 = test_function3(f2);
    
    if (f1 > f2) {
        gl_FragColor = v1 + v2 + v3 + v4;
    } else
        gl_FragColor = v1 + v2 + v3 + v4;
}

float f4 = 5.5;
vec4 test_function3(float ff)
{
    if (ff > f4) 
	return vec4(ff);
    else
        return vec4(f4);
}

float f3 = 4.5;
vec4 test_function2(float ff)
{
    vec4 func_vec4 = vec4(ff+f3);
    return func_vec4;
}

float f5 = 6.5;
