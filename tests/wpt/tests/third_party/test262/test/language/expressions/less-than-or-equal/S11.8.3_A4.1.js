// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is NaN, return false (if result in 11.8.5 is undefined, return false)
es5id: 11.8.3_A4.1
description: y is number primitive
---*/

//CHECK#1
if ((Number.NaN <= 0) !== false) {
  throw new Test262Error('#1: (NaN <= 0) === false');
}

//CHECK#2
if ((Number.NaN <= 1.1) !== false) {
  throw new Test262Error('#2: (NaN <= 1.1) === false');
}

//CHECK#3
if ((Number.NaN <= -1.1) !== false) {
  throw new Test262Error('#3: (NaN <= -1.1) === false');
}

//CHECK#4
if ((Number.NaN <= Number.NaN) !== false) {
  throw new Test262Error('#4: (NaN <= NaN) === false');
}

//CHECK#5
if ((Number.NaN <= Number.POSITIVE_INFINITY) !== false) {
  throw new Test262Error('#5: (NaN <= +Infinity) === false');
}

//CHECK#6
if ((Number.NaN <= Number.NEGATIVE_INFINITY) !== false) {
  throw new Test262Error('#6: (NaN <= -Infinity) === false');
}

//CHECK#7
if ((Number.NaN <= Number.MAX_VALUE) !== false) {
  throw new Test262Error('#7: (NaN <= Number.MAX_VALUE) === false');
}

//CHECK#8
if ((Number.NaN <= Number.MIN_VALUE) !== false) {
  throw new Test262Error('#8: (NaN <= Number.MIN_VALUE) === false');
}
