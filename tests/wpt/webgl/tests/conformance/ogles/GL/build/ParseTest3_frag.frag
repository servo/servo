
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
    const vec4 v = vec4(normalize(vec4(1)));    // Builtin functions are constant expressions if all their parameters are constant expressions - code ok
    const vec4 v1 = vec4(clamp(1.0, .20, 3.0)); // Builtin functions are constant expressions if all their parameters are constant expressions - code ok
    float f = 1.0;
    const vec4 v2 = vec4(float(vec4(1,2,3,f))); // f is not constant - code fails and test does not compile (expected)

    gl_FragColor = v + v1 + v2;
}
