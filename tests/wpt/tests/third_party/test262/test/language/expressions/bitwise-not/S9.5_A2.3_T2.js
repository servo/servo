// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If result is greater than or equal to 2^31, return result -2^32
es5id: 9.5_A2.3_T2
description: Use operator ~
---*/

// CHECK#1
if (~2147483647 !== -2147483648) {
  throw new Test262Error('#1: ~2147483647 ==== -2147483648)');
}

// CHECK#2
if (~2147483648 !== ~-2147483648) {
  throw new Test262Error('#2: ~2147483648 ==== ~-2147483648)');
}

// CHECK#3
if (~2147483649 !== ~-2147483647) {
  throw new Test262Error('#3: ~2147483649 ==== ~-2147483647)');
}

// CHECK#4
if (~4294967295 !== ~-1) {
  throw new Test262Error('#4: ~4294967295 ==== ~-1)');
}

// CHECK#5
if (~4294967296 !== ~0) {
  throw new Test262Error('#5: ~4294967296 ==== ~0)');
}

// CHECK#6
if (~4294967297 !== ~1) {
  throw new Test262Error('#6: ~4294967297 ==== ~1)');
}
