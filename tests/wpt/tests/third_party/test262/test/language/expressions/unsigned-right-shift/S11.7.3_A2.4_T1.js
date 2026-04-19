// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.7.3_A2.4_T1
description: Checking with "="
---*/

//CHECK#1
var x = 0; 
if ((x = 1) >>> x !== 0) {
  throw new Test262Error('#1: var x = 0; (x = 1) >>> x === 0. Actual: ' + ((x = 1) >>> x));
}

//CHECK#2
var x = -4; 
if (x >>> (x = 1) !== 2147483646) {
  throw new Test262Error('#2: var x = -4; x >>> (x = 1) === 2147483646. Actual: ' + (x >>> (x = 1)));
}
