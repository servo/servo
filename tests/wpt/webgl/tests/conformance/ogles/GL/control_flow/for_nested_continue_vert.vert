
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
	int val1 = 0, val2 = 0;
    	for(int i=0;i<10;i++)
	{
	  	count1++;
		count2 = 0;
		for(int j=0;j<10;j++)
		{
			count2++;
			if(count2 == 5)
				continue;
			else
				val2 += count2;

		}


	  	if(count1 == 5)
            		continue;
	  	else
	    		val1 += count1;

	}
	float gray;
	if( (val1 == 50) && (val2 == 500) )
	gray=1.0;
	else gray=0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
