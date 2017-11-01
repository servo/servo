
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
uniform float	mortarThickness;
uniform vec3	brickColor;
uniform vec3	mortarColor;

uniform float	brickMortarWidth;
uniform float	brickMortarHeight;
uniform float	mwf;
uniform float	mhf;

varying vec3  Position;
varying float lightIntensity;

void main (void)
{
    vec3	ct;
    float	ss, tt, w, h;

    vec3 pos = Position;
    
    ss = pos.x / brickMortarWidth;
    tt = pos.z / brickMortarHeight;

    if (fract (tt * 0.5) > 0.5)
        ss += 0.5;

    ss = fract (ss);
    tt = fract (tt);

    w = step (mwf, ss) - step (1.0 - mwf, ss);
    h = step (mhf, tt) - step (1.0 - mhf, tt);

    ct = clamp(mix (mortarColor, brickColor, w * h) * lightIntensity, 0.0, 1.0);

    gl_FragColor = vec4 (ct, 1.0);
}
