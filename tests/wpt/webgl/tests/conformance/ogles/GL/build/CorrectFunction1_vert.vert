
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


vec3 bar(vec3, vec3);

uniform vec2 v;

bool foo(out vec3);

void main()
{
    bool b1, b2, b3, b4, b5, b6;

    b1 = any(lessThan(v, v));

    b2 = all(lessThanEqual(v, v));

    b3 = any(not(greaterThan(v, v)));

    b4 = any(greaterThanEqual(v, v));

    b5 = any(notEqual(v, v));

    b6 = any(equal(v, v));

    vec2 u;
    if (b1 && b2 && b3 && b4 && b5 && b6)
        u = v;

    gl_Position = vec4(u, u);
}
