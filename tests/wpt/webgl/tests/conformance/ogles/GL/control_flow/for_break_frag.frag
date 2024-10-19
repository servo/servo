
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

void main (void)
{
	int count = 0;
        for(int i=0;i<5;i++)
	{
	  count++;
	  if(count == 3)
            break;
	}

	float gray;
	if( count == 3)
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
