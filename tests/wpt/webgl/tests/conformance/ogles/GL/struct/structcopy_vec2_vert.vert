
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
	vec2 a;
	vec2 b;
};



void main (void)
{
	sabcd s1 = sabcd(vec2(12.0, 29.0), vec2(13.0, 26.0) );
	sabcd s2 = sabcd(vec2(0.0, 0.0), vec2(0.0, 0.0) );
	s2 = s1;
	color = vec4( vec3(  (s2.a[0] + s2.a[1] + s2.b[0] + s2.b[1]) / 80.0  ), 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
