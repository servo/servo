// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.5.2_A2.4_T3
description: Checking with undeclarated variables
---*/

//CHECK#1
try {
  x / (x = 1);
  throw new Test262Error('#1.1: x / (x = 1) throw ReferenceError. Actual: ' + (x / (x = 1)));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: x / (x = 1) throw ReferenceError. Actual: ' + (e));  
  }
}
