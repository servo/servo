
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	int count1 = 0, count2 = 0;
        for(int i=0;i<45;i++)
	{
	  count1++;
	  count2 = 0;
	  for(int j=0;j<45;j++)
	  {
	     count2++;
	     if(count2 == 29)
		break;
	  }
	  if(count1 == 29)
            break;
	}
	float gray;
	if( (count1 == 29) && (count2 == 29))
	gray=1.0;
	else gray=0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
