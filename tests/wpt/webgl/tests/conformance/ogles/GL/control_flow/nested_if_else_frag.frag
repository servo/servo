
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
	int setval = 30;

	if(--setval!=29)
		if( (setval+=11) == 40)
			if(setval/4 == 11)
				setval = 11;
			else if(setval/4 == 10)
				if(setval-3 == 37)
					setval=12;
				else setval = 9;
			else setval = 10;
		else setval = 30;
	else if(setval == 29)
		if((setval+=19) != 48)
			setval = 13;
		else if((setval+=19) == 29)
			setval = 28;
		else setval = 53;
	else setval = 32;
	float gray;
	if( setval == 53 )
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
