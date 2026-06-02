
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct s {
    float f;
    vec3 v;
};

void main()
{
    const vec4 v = (vec4(1,2,3,4), vec4(5,6,7,8));  // 5,6,7,8
    const s s1 = (s(9.0, vec3(10,11,12)), s(13.0, vec3(14,15,16))); // 13,14,15,16
    gl_FragColor = v + vec4(s1.f, s1.v);  // 18, 20, 22, 24
}
