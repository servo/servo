// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a floating-point multiplication is governed by the rules of
    IEEE 754 double-precision arithmetics
es5id: 11.5.1_A4_T3
description: Multiplication of an infinity by a zero results in NaN
---*/

//CHECK#1
if (isNaN(Number.NEGATIVE_INFINITY * 0) !== true) {
  throw new Test262Error('#1: Infinity * 0 === Not-a-Number. Actual: ' + (Infinity * 0));
}

//CHECK#2
if (isNaN(-0 * Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#2: -0 * -Infinity === Not-a-Number. Actual: ' + (-0 * -Infinity));
}

//CHECK#3
if (isNaN(Number.POSITIVE_INFINITY * -0) !== true) {
  throw new Test262Error('#3: Infinity * -0 === Not-a-Number. Actual: ' + (Infinity * -0));
}

//CHECK#4
if (isNaN(0 * Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#4: 0 * Infinity === Not-a-Number. Actual: ' + (0 * Infinity));
}

//CHECK#5
if (isNaN(Number.NEGATIVE_INFINITY * -0) !== true) {
  throw new Test262Error('#5: Infinity * -0 === Not-a-Number. Actual: ' + (Infinity * -0));
}

//CHECK#6
if (isNaN(0 * Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#6: 0 * -Infinity === Not-a-Number. Actual: ' + (0 * -Infinity));
}

//CHECK#7
if (isNaN(Number.POSITIVE_INFINITY * 0) !== true) {
  throw new Test262Error('#7: Infinity * 0 === Not-a-Number. Actual: ' + (Infinity * 0));
}

//CHECK#8
if (isNaN(-0 * Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#8: -0 * Infinity === Not-a-Number. Actual: ' + (-0 * Infinity));
}
