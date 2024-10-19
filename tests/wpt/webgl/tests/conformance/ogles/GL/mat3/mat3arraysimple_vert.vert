
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
//
// mat3arraysimple_vert.vert: Vertex shader solid color  testing indirect referencing into uniforms
// The vec3 values are determined at runtime.
//
//

uniform mat3 testmat3[2];
varying vec4  color;


void main(void)
{
     vec3 result = vec3(0.0, 0.0, 0.0);

     result = testmat3[1][0] + testmat3[1][1] + testmat3[1][2];

     color = vec4(result/2.0, 0.5);

     gl_Position     = gtf_ModelViewProjectionMatrix * gtf_Vertex;


}
