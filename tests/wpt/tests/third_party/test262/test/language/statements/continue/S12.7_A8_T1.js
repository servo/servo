// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Appearing of "continue" within a "try/catch" Block yields SyntaxError
es5id: 12.7_A8_T1
description: >
    Checking if execution of "continue Identifier" within catch Block
    fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

try{
} catch(e){
	continue LABEL2;
	LABEL2 : do {
	} while(0);
};
