// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of "break" within "try/catch" Block yields SyntaxError
es5id: 12.8_A8_T2
description: >
    Checking if using "break Identifier" from within catch Block
    appears to be invalid
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

try{
} catch(e){
	break;
	LABEL2 : do {
	} while(0);
}
