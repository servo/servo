// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are Number-s minus NaN, +0, -0.
    Return true, if x is the same number value as y; otherwise, return false
es5id: 11.9.1_A4.3
description: x and y are primitive numbers
---*/

//CHECK#1
if ((Number.POSITIVE_INFINITY == Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#1: (+Infinity == +Infinity) === true');
}

//CHECK#2
if ((Number.NEGATIVE_INFINITY == Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#2: (-Infinity == -Infinity) === true');
}

//CHECK#3
if ((Number.POSITIVE_INFINITY == -Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#3: (+Infinity == -(-Infinity)) === true');
}

//CHECK#4
if ((1 == 0.999999999999) !== false) {
  throw new Test262Error('#4: (1 == 0.999999999999) === false');
}

//CHECK#5
if ((1.0 == 1) !== true) {
  throw new Test262Error('#5: (1.0 == 1) === true');
}
