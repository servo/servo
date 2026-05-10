// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.8.2_A2.4_T1
description: Checking with "="
---*/

//CHECK#1
var x = 0; 
if ((x = 1) > x !== false) {
  throw new Test262Error('#1: var x = 0; (x = 1) > x === false');
}

//CHECK#2
var x = 1; 
if (x > (x = 0) !== true) {
  throw new Test262Error('#2: var x = 1; x > (x = 0) === true');
}
