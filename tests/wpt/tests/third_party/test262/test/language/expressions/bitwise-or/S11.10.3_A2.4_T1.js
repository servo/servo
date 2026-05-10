// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.10.3_A2.4_T1
description: Checking with "="
---*/

//CHECK#1
var x = 1; 
if (((x = 0) | x) !== 0) {
  throw new Test262Error('#1: var x = 1; ((x = 0) | x) === 0. Actual: ' + (((x = 0) | x)));
}

//CHECK#2
var x = 1; 
if ((x | (x = 0)) !== 1) {
  throw new Test262Error('#2: var x = 1; (x | (x = 0)) === 1. Actual: ' + ((x | (x = 0))));
}
