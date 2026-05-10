// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x @= y uses PutValue(x, x @ y)
es5id: 11.13.2_A3.1_T7
description: Checking Expression and Variable statements for x >>= y
---*/

//CHECK#1
var x = 4;
x >>= 1; 
if (x !== 2) {
  throw new Test262Error('#1: var x = 4; x >>= 1; x === 2. Actual: ' + (x));
}

//CHECK#2
var y;
y = 4;
y >>= 1;
if (y !== 2) {
  throw new Test262Error('#2: y = 4; y >>= 1; y === 2. Actual: ' + (y));
}
