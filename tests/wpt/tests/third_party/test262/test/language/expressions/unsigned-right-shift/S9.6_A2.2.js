// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Compute result modulo 2^32
es5id: 9.6_A2.2
description: Use operator >>>0
---*/

// CHECK#1
if ((-2147483647 >>> 0) !== 2147483649) {
  throw new Test262Error('#1: (-2147483647 >>> 0) === 2147483649. Actual: ' + ((-2147483647 >>> 0)));
}

// CHECK#2
if ((-2147483648 >>> 0) !== 2147483648) {
  throw new Test262Error('#2: (-2147483648 >>> 0) === 2147483648. Actual: ' + ((-2147483648 >>> 0)));
}

// CHECK#3
if ((-2147483649 >>> 0) !== 2147483647) {
  throw new Test262Error('#3: (-2147483649 >>> 0) === 2147483647. Actual: ' + ((-2147483649 >>> 0)));
}

// CHECK#4
if ((-4294967295 >>> 0) !== 1) {
  throw new Test262Error('#4: (-4294967295 >>> 0) === 1. Actual: ' + ((-4294967295 >>> 0)));
}

// CHECK#5
if ((-4294967296 >>> 0) !== 0) {
  throw new Test262Error('#5: (-4294967296 >>> 0) === 0. Actual: ' + ((-4294967296 >>> 0)));
}

// CHECK#6
if ((-4294967297 >>> 0) !== 4294967295) {
  throw new Test262Error('#6: (-4294967297 >>> 0) === 4294967295. Actual: ' + ((-4294967297 >>> 0)));
}

// CHECK#7
if ((4294967295 >>> 0) !== 4294967295) {
  throw new Test262Error('#7: (4294967295 >>> 0) === 4294967295. Actual: ' + ((4294967295 >>> 0)));
}

// CHECK#8
if ((4294967296 >>> 0) !== 0) {
  throw new Test262Error('#8: (4294967296 >>> 0) === 0. Actual: ' + ((4294967296 >>> 0)));
}

// CHECK#9
if ((4294967297 >>> 0) !== 1) {
  throw new Test262Error('#9: (4294967297 >>> 0) === 1. Actual: ' + ((4294967297 >>> 0)));
}

// CHECK#10
if ((8589934591 >>> 0) !== 4294967295) {
  throw new Test262Error('#10: (8589934591 >>> 0) === 4294967295. Actual: ' + ((8589934591 >>> 0)));
}

// CHECK#11
if ((8589934592 >>> 0) !== 0) {
  throw new Test262Error('#11: (8589934592 >>> 0) === 0. Actual: ' + ((8589934592 >>> 0)));
}

// CHECK#12
if ((8589934593 >>> 0) !== 1) {
  throw new Test262Error('#12: (8589934593 >>> 0) === 1. Actual: ' + ((8589934593 >>> 0)));
}
