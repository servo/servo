// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check examples for automatic semicolon insertion from the standard
es5id: 7.9.2_A1_T4
description: >
    return \n a+b is a valid sentence in the ECMAScript grammar  with
    automatic semicolon insertion, but returned undefined
---*/

//CHECK#1
var a=1,b=2;
function test(){
	return
	a+b
}
var x=test();
if (x!==undefined) throw new Test262Error('#1: Automatic semicolon insertion not work with return');
