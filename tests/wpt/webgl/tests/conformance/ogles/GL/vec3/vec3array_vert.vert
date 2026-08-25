
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
//
// vec3array_vert.vert: Simple vertex shader using vec3 to get colors.
//
//

varying vec4 color;
uniform vec3 lightPosition[2];

void main(void)
{
    vec3 v[2];

    v[1] = vec3(gtf_Color.r, gtf_Color.g, gtf_Color.b);

    v[0] = lightPosition[1];

    color =  vec4(v[1] + v[1], 0.0)/2.0;
    gl_Position     = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
