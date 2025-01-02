
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
    vec2 v = vec2(1,5);
    // at the end of next statement, values in
    // v.x = 12, v.y = 12
    v.xy += v.yx += v.xy;
    // v1 and v2, both are initialized with 12
    vec2 v1 = v, v2 = v;

    v1.xy += v2.yx += ++(v.xy);  // v1 = 37, v2 = 25 each
    v1.xy += v2.yx += (v.xy)++;  // v1 = 75, v2 = 38 each
    gl_FragColor = vec4(v1,v2);  // 75, 75, 38, 38
}
