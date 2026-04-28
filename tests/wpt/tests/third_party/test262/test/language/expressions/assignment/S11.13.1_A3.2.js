// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x = y returns GetValue(y)
es5id: 11.13.1_A3.2
description: Checking Expression and Variable statements
---*/

//CHECK#1
var x = 0;
if ((x = 1) !== 1) {
  throw new Test262Error('#1: var x = 0; (x = 1) === 1. Actual: ' + ((x = 1)));
}

//CHECK#2
x = 0;
if ((x = 1) !== 1) {
  throw new Test262Error('#2: x = 0; (x = 1) === 1. Actual: ' + ((x = 1)));
}
