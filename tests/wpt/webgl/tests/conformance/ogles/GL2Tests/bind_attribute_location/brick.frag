
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
