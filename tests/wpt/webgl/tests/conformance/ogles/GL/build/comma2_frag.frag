
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void main()
{
    const vec4 v = (vec4(1,2,3,4), vec4(5,6,7,8), 1.2); // right most value of comma operator shoul be a vec4
    const vec4 v1 = (vec3(0.2, 2.0), vec4(1,2,3,4), vec4(5,6,7,8));
    const vec4 v2 = (vec4(1,2,3,4), vec2(2.1, 2),  vec4(5,6,7,8));
    gl_FragColor = v + v1 + v2;
}
