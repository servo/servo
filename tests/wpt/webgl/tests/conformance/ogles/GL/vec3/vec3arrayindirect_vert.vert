
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
//
// vec3arrayindirect_vert.vert: Vertex shader solid color
// The vec3 values are determined at runtime.
//
//

uniform vec3 lightPosition[2];
varying vec4  color;

void main(void)
{
     color = vec4(0.0);

     for (int i = 0; i < 2; i++)
     {
          color += vec4(lightPosition[i], 0.0);
     }

     color /= 2.0;

     gl_Position     = gtf_ModelViewProjectionMatrix * gtf_Vertex;


}
