// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator uses floor, abs
es5id: 9.5_A3.2_T1
description: Use operator <<0
---*/

// CHECK#1
if ((1.2345 << 0) !== 1) {
  throw new Test262Error('#1: (1.2345 << 0) === 1. Actual: ' + ((1.2345 << 0)));
}

// CHECK#2
if ((-5.4321 << 0) !== -5) {
  throw new Test262Error('#2: (-5.4321 << 0) === -5. Actual: ' + ((-5.4321 << 0)));
}
