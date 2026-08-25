
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
