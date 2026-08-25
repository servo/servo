// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.10.2_A2.4_T3
description: Checking with undeclarated variables
flags: [noStrict]
---*/

//CHECK#1
try {
  x ^ (x = 1);
  throw new Test262Error('#1.1: x ^ (x = 1) throw ReferenceError. Actual: ' + (x ^ (x = 1)));
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: x ^ (x = 1) throw ReferenceError. Actual: ' + (e));
  }
}

//CHECK#2
if (((y = 1) ^ y) !== 0) {
  throw new Test262Error('#2: ((y = 1) ^ y) === 0. Actual: ' + (((y = 1) ^ y)));
}
