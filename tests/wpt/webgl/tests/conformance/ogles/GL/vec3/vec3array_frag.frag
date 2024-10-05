
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
//
// vec3array_frag.frag: Simple Fragment shader using vec3 to get colors.
//
//

varying vec4 color;

uniform vec3 lightPosition[2];

void main(void)
{
    vec3 v[2];

    v[1] = vec3(color.r, color.g, color.b);


    v[0] = lightPosition[1];


    gl_FragColor =  vec4(v[1] + v[1], 0.0)/2.0;
}
