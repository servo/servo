
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
vec2 func()
{
    vec2 v;
    return v;
}

void main()
{
    const vec3 v = vec3(1.0, func()); // user defined functions do not return const value
    gl_FragColor = vec4(v, v);
}
