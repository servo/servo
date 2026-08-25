// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Negating +0 produces -0, negating -0 produces +0
es5id: 11.4.7_A4.2
description: Checking Infinity
---*/

//CHECK#1
var x = 0; 
x = -x;
if (x !== -0) {
  throw new Test262Error('#1.1: var x = 0; x = -x; x === 0. Actual: ' + (x));
} else {
  if (1/x !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#1.2: var x = 0; x = -x; x === - 0. Actual: +0');
  }
}

//CHECK#2
var x = -0; 
x = -x;
if (x !== 0) {
  throw new Test262Error('#2.1: var x = -0; x = -x; x === 0. Actual: ' + (x));
} else {
  if (1/x !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#2.2: var x = -0; x = -x; x === + 0. Actual: -0');
  }
}
