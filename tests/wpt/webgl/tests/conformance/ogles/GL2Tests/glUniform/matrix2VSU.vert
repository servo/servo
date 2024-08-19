
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
attribute vec4 gtf_Color;
uniform mat4 gtf_ModelViewProjectionMatrix;
uniform mat4 transforms;
uniform mat4 anotherMatrix;

varying vec4 color;

void main(void)
{
  color = gtf_Color; // color is per vertex and matches glColor already used by Vertex

   gl_Position = gtf_ModelViewProjectionMatrix* transforms * anotherMatrix * gtf_Vertex;
}