// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a floating-point multiplication is governed by the rules of
    IEEE 754 double-precision arithmetics
es5id: 11.5.1_A4_T6
description: >
    If the magnitude is too large to represent, the result is then an
    infinity of appropriate sign
---*/

//CHECK#1
if (Number.MAX_VALUE * 1.1 !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#1: Number.MAX_VALUE * 1.1 === Number.POSITIVE_INFINITY. Actual: ' + (Number.MAX_VALUE * 1.1));
}

//CHECK#2
if (-1.1 * Number.MAX_VALUE !== Number.NEGATIVE_INFINITY) {
  throw new Test262Error('#2: -1.1 * Number.MAX_VALUE === Number.NEGATIVE_INFINITY. Actual: ' + (-1.1 * Number.MAX_VALUE));
} 

//CHECK#3
if (Number.MAX_VALUE * 1 !== Number.MAX_VALUE) {
  throw new Test262Error('#3: Number.MAX_VALUE * 1 === Number.MAX_VALUE. Actual: ' + (Number.MAX_VALUE * 1));
}

//CHECK#4
if (-1 * Number.MAX_VALUE !== -Number.MAX_VALUE) {
  throw new Test262Error('#4: -1 * Number.MAX_VALUE === -Number.MAX_VALUE. Actual: ' + (-1 * Number.MAX_VALUE));
}
