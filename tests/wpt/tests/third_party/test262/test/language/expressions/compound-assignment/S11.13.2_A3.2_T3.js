// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x @= y returns x @ y
es5id: 11.13.2_A3.2_T3
description: Checking Expression and Variable statements for x %= y
---*/

//CHECK#1
var x = -1;
var x1 = (x %= 2);
if (x1 !== -1) {
  throw new Test262Error('#1: var x = -1; var x1 = (x %= 2); x1 === -1. Actual: ' + (x1));
}

//CHECK#2
var y;
var y1;
y = -1;
y1 = (y %= 2);
if (y1 !== -1) {
  throw new Test262Error('#2: y = -1; y1 = (y %= 2); y1 === -1. Actual: ' + (y1));
}
