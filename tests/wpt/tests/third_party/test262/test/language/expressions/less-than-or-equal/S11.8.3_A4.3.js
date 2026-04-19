// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x and y are the same number value, return true
es5id: 11.8.3_A4.3
description: x and y are number primitives
---*/

//CHECK#1
if ((1 <= 1) !== true) {
  throw new Test262Error('#1: (1 <= 1) === true');
}

//CHECK#2
if ((1.1 <= 1.1) !== true) {
  throw new Test262Error('#2: (1.1 <= 1.1) === true');
}

//CHECK#3
if ((-1.1 <= -1.1) !== true) {
  throw new Test262Error('#3: (-1.1 <= -1.1) === true');
}

//CHECK#4
if ((Number.NEGATIVE_INFINITY <= Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#4: (-Infinity <= -Infinity) === true');
}

//CHECK#5
if ((Number.POSITIVE_INFINITY <= Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#5: (+Infinity <= +Infinity) === true');
}

//CHECK#6
if ((Number.MAX_VALUE <= Number.MAX_VALUE) !== true) {
  throw new Test262Error('#6: (Number.MAX_VALUE <= Number.MAX_VALUE) === true');
}

//CHECK#7
if ((Number.MIN_VALUE <= Number.MIN_VALUE) !== true) {
  throw new Test262Error('#7: (Number.MIN_VALUE <= Number.MIN_VALUE) === true');
}
