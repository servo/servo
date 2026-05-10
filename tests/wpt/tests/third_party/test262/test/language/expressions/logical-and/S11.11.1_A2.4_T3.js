// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.11.1_A2.4_T3
description: Checking with undeclarated variables
flags: [noStrict]
---*/

//CHECK#1
try {
  x && (x = true);
  throw new Test262Error('#1.1: x && (x = true) throw ReferenceError. Actual: ' + (x && (x = true)));
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: x && (x = true) throw ReferenceError. Actual: ' + (e));
  }
}

//CHECK#2
if (((y = true) && y) !== true) {
  throw new Test262Error('#2: ((y = true) && y) === true');
}
