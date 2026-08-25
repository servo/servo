// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator uses floor, abs
es5id: 9.5_A3.2_T2
description: Use operator ~
---*/

// CHECK#1
if (~1.2345 !== ~1) {
  throw new Test262Error('#1: ~1.2345 === ~1)');
}

// CHECK#2
if (~-5.4321 !== ~-5) {
  throw new Test262Error('#2: ~-5.4321 === ~-5)');
}
