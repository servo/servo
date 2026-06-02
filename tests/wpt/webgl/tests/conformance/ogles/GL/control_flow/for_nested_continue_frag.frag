
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
	int count1 = 0, count2 = 0;
	int val1 = 0, val2 = 0;
    	for(int i=0;i<4;i++)
	{
	  	count1++;
		count2 = 0;
		for(int j=0;j<4;j++)
		{
			count2++;
			if(count2 == 2)
				continue;
			else
				val2 += count2;

		}


	  	if(count1 == 2)
            		continue;
	  	else
	    		val1 += count1;

	}
	float gray;
	if( (val1 == 8) && (val2 == 32) )
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
