
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
