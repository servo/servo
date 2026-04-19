// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x++ returns ToNumber(x)
es5id: 11.3.1_A4_T1
description: Type(x) is boolean primitive or Boolean object
---*/

//CHECK#1
var x = false;
var y = x++;
if (y !== 0) {
  throw new Test262Error('#1: var x = false; var y = x++; y === 0. Actual: ' + (y));
}

//CHECK#2
var x = new Boolean(true);
var y = x++;
if (y !== 1) {
  throw new Test262Error('#2: var x = new Boolean(true); var y = x++; y === 1. Actual: ' + (y));
}
