
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
	bool a;
	bool b;
	bool c;
	bool d;
};



void main (void)
{
	sabcd s1 = sabcd(bool(12), bool(0), bool(25.5), bool(0.0));
	sabcd s2 = sabcd(bool(0.0), bool(0.0), bool(0.0), bool(0.0));
	s2 = s1;
	float gray = 0.0;
	if( (s2.a==true) && (s2.b==false) && (s2.c == true) && (s2.d==false))
	  gray=1.0;
	else
          gray =0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;

}
