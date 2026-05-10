// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check examples for automatic semicolon insertion from the standard
es5id: 7.9.2_A1_T7
description: >
    a=b+c \n (d+e).print() is a valid sentence in the ECMAScript
    grammar,  and automatic semicolon insertion not run
---*/

//CHECK#1
function c (a){
	return 2*a;
}

var a=1,b=2,d=4,e=5;

a=b+c
(d+e)

if (a !== 20) throw new Test262Error('#1: Automatic semicolon insertion work wrong');
