// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Appearing of "break" within eval statement that is nested in an
    IterationStatement yields SyntaxError
es5id: 12.8_A7
description: Using eval "eval("break LABEL1")"
---*/

var x=0,y=0;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try{
	LABEL1 : do {
        x++;
        eval("break LABEL1");
        y++;
    } while(0);
	throw new Test262Error('#1: eval("break LABEL1") does not lead to throwing exception');
} catch(e){
	if(!(e instanceof SyntaxError)){
		throw new Test262Error("1.1: Appearing of break within eval statement inside of IterationStatement yields SyntaxError");
	}
}
//
//////////////////////////////////////////////////////////////////////////////
