// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a floating-point multiplication is governed by the rules of
    IEEE 754 double-precision arithmetics
es5id: 11.5.1_A4_T7
description: >
    If the magnitude is too small to represent, the result is then a
    zero of appropriate sign
---*/

//CHECK#1
if (Number.MIN_VALUE * 0.1 !== 0) {
  throw new Test262Error('#1: Number.MIN_VALUE * 0.1 === 0. Actual: ' + (Number.MIN_VALUE * 0.1));
}

//CHECK#2
if (-0.1 * Number.MIN_VALUE !== -0) {
  throw new Test262Error('#2.1: -0.1 * Number.MIN_VALUE === -0. Actual: ' + (-0.1 * Number.MIN_VALUE));
} else {
  if (1 / (-0.1 * Number.MIN_VALUE) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#2.2: -0.1 * Number.MIN_VALUE === -0. Actual: +0');
  }
}

//CHECK#3
if (Number.MIN_VALUE * 0.5 !== 0) {
  throw new Test262Error('#3: Number.MIN_VALUE * 0.5 === 0. Actual: ' + (Number.MIN_VALUE * 0.5));
}

//CHECK#4
if (-0.5 * Number.MIN_VALUE !== -0) {
  throw new Test262Error('#4.1: -0.5 * Number.MIN_VALUE === -0. Actual: ' + (-0.5 * Number.MIN_VALUE));
} else {
  if (1 / (-0.5 * Number.MIN_VALUE) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#4.2: -0.5 * Number.MIN_VALUE === -0. Actual: +0');
  }
}

//CHECK#5
if (Number.MIN_VALUE * 0.51 !== Number.MIN_VALUE) {
  throw new Test262Error('#5: Number.MIN_VALUE * 0.51 === Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE * 0.51));
}

//CHECK#6
if (-0.51 * Number.MIN_VALUE !== -Number.MIN_VALUE) {
  throw new Test262Error('#6: -0.51 * Number.MIN_VALUE === -Number.MIN_VALUE. Actual: ' + (-0.51 * Number.MIN_VALUE));
}

//CHECK#7
if (Number.MIN_VALUE * 0.9 !== Number.MIN_VALUE) {
  throw new Test262Error('#7: Number.MIN_VALUE * 0.9 === Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE * 0.9));
}

//CHECK#8
if (-0.9 * Number.MIN_VALUE !== -Number.MIN_VALUE) {
  throw new Test262Error('#8: -0.9 * Number.MIN_VALUE === -Number.MIN_VALUE. Actual: ' + (-0.9 * Number.MIN_VALUE));
}
