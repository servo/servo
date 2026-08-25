// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Only constructor call (with "new" keyword) makes instance
es5id: 11.8.6_A4_T3
description: Checking String case
---*/

//CHECK#1
if ("" instanceof String) {
	throw new Test262Error('#1: "" is not instanceof String');
}

//CHECK#2
if (String("") instanceof String) {
	throw new Test262Error('#2: String("") is not instanceof String');
}

//CHECK#3
if (new String instanceof String !== true) {
	throw new Test262Error('#3: new String instanceof String');
}
