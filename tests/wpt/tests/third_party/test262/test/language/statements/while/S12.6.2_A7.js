// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The "while" Statement is evaluted according to 12.6.2 and returns
    (normal, V, empty)
es5id: 12.6.2_A7
description: using eval
---*/

var __evaluated;
var __condition=0

__evaluated = eval("while (__condition<5) eval(\"__condition++\");");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__condition !== 5) {
	throw new Test262Error('#1: The "while" statement is evaluated as described in the Standard');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__evaluated !== 4) {
	throw new Test262Error('#2: The "while" statement returns (normal, V, empty)');
}
//
//////////////////////////////////////////////////////////////////////////////
