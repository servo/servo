// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x++ returns ToNumber(x)
es5id: 11.3.1_A4_T4
description: Type(x) is undefined or null
---*/

//CHECK#1
var x;
var y = x++;
if (isNaN(y) !== true) {
  throw new Test262Error('#1: var x; var y = x++; y === Not-a-Number. Actual: ' + (y));
}

//CHECK#2
var x = null;
var y = x++;
if (y !== 0) {
  throw new Test262Error('#2: var x = null; var y = x++; y === 0. Actual: ' + (y));
}
