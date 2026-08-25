
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
attribute float myAttribute1;
attribute float myAttribute2;
attribute float myAttribute3;

varying vec3 colors;

void main(void) 
{
	colors = vec3(myAttribute1, 0, 0);
    
    gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
