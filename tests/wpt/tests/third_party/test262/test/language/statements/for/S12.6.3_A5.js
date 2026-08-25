// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    "in"-expression wrapped into "eval" statement is allowed as a
    ExpressionNoIn in "for (ExpressionNoIn; FirstExpression;
    SecondExpression) Statement" IterationStatement
es5id: 12.6.3_A5
description: Using eval "for(eval("i in arr");1;)"
---*/

var arr, i;

arr = [1,2,3,4,5];
i = 1;
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	for(eval("i in arr");1;) {break;};	
} catch (e) {	
		throw new Test262Error('#1.1: for(eval("i in arr");1;) {break;}; does not lead to throwing exception');	
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
try {
	for(eval("var i = 1 in arr");1;) {break;};	
} catch (e) {	
		throw new Test262Error('#2.1: for(eval("var i = 1 in arr");1;) {break;}; does not lead to throwing exception');	
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
try {
	for(eval("1 in arr");1;) {break;};
} catch (e) {	
		throw new Test262Error('#3.1: for(eval("1 in arr");1;) {break;}; does not lead to throwing exception');	
}
//
//////////////////////////////////////////////////////////////////////////////
