
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;



struct sabcd
{
	vec3 a;
	vec3 b;
};



void main (void)
{
	sabcd s = sabcd(vec3(12.0, 29.0, 32.0), vec3(13.0, 26.0, 38.0 ) );
	color = vec4( vec3(  (s.a[0] + s.a[1] + s.a[2] + s.b[0] + s.b[1] + s.b[2]) / 150.0  ), 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
