// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of division is determined by the specification of IEEE 754
    arithmetics
es5id: 11.5.2_A4_T10
description: >
    If both operands are finite and nonzero, the quotient is computed
    and rounded using IEEE 754 round-to-nearest mode.  If the
    magnitude is too small to represent, the result is then a zero of
    appropriate sign
---*/

//CHECK#1
if (Number.MIN_VALUE / 2.1 !== 0) {
  throw new Test262Error('#1: Number.MIN_VALUE / 2.1 === 0. Actual: ' + (Number.MIN_VALUE / 2.1));
}

//CHECK#2
if (Number.MIN_VALUE / -2.1 !== -0) {
  throw new Test262Error('#2.1: Number.MIN_VALUE / -2.1 === 0. Actual: ' + (Number.MIN_VALUE / -2.1));
} else {
  if (1 / (Number.MIN_VALUE / -2.1) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#2.2: Number.MIN_VALUE / -2.1 === -0. Actual: +0');
  }
}

//CHECK#3
if (Number.MIN_VALUE / 2.0 !== 0) {
  throw new Test262Error('#3: Number.MIN_VALUE / 2.0 === 0. Actual: ' + (Number.MIN_VALUE / 2.0));
}

//CHECK#4
if (Number.MIN_VALUE / -2.0 !== -0) {
  throw new Test262Error('#4.1: Number.MIN_VALUE / -2.0 === -0. Actual: ' + (Number.MIN_VALUE / -2.0));
} else {
  if (1 / (Number.MIN_VALUE / -2.0) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#4.2: Number.MIN_VALUE / -2.0 === -0. Actual: +0');
  }
}

//CHECK#5
if (Number.MIN_VALUE / 1.9 !== Number.MIN_VALUE) {
  throw new Test262Error('#5: Number.MIN_VALUE / 1.9 === Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE / 1.9));
}

//CHECK#6
if (Number.MIN_VALUE / -1.9 !== -Number.MIN_VALUE) {
  throw new Test262Error('#6: Number.MIN_VALUE / -1.9 === -Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE / -1.9));
}

//CHECK#7
if (Number.MIN_VALUE / 1.1 !== Number.MIN_VALUE) {
  throw new Test262Error('#7: Number.MIN_VALUE / 1.1 === Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE / 1.1));
}

//CHECK#8
if (Number.MIN_VALUE / -1.1 !== -Number.MIN_VALUE) {
  throw new Test262Error('#8: Number.MIN_VALUE / -1.1 === -Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE / -1.1));
}
