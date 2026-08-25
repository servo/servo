// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are Number-s minus NaN, +0, -0.
    Return false, if x is the same number value as y; otherwise, return true
es5id: 11.9.2_A4.3
description: x and y are primitive numbers
---*/

//CHECK#1
if ((Number.POSITIVE_INFINITY != Number.POSITIVE_INFINITY) !== false) {
  throw new Test262Error('#1: (+Infinity != +Infinity) === false');
}

//CHECK#2
if ((Number.NEGATIVE_INFINITY != Number.NEGATIVE_INFINITY) !== false) {
  throw new Test262Error('#2: (-Infinity != -Infinity) === false');
}

//CHECK#3
if ((Number.POSITIVE_INFINITY != -Number.NEGATIVE_INFINITY) !== false) {
  throw new Test262Error('#3: (+Infinity != -(-Infinity)) === false');
}

//CHECK#4
if ((1 != 0.999999999999) !== true) {
  throw new Test262Error('#4: (1 != 0.999999999999) === true');
}

//CHECK#5
if ((1.0 != 1) !== false) {
  throw new Test262Error('#5: (1.0 != 1) === false');
}
