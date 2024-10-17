
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
attribute vec4 gtf_MultiTexCoord0;

varying vec4 texCoord[1];
varying vec4 color;

uniform mat4 gtf_ModelViewProjectionMatrix;

void main (void)
{
    color = gtf_Color;
    texCoord[0] = gtf_MultiTexCoord0;
    gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
