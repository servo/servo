// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a ECMAScript floating-point remainder operation is
    determined by the rules of IEEE arithmetics
es5id: 11.5.3_A4_T4
description: If the divisor is zero results is NaN
---*/

//CHECK#1
if (isNaN(-0 % 0) !== true) {
  throw new Test262Error('#1: -0 % 0 === Not-a-Number. Actual: ' + (-0 % 0));
}

//CHECK#2
if (isNaN(-0 % -0) !== true) {
  throw new Test262Error('#2: -0 % -0 === Not-a-Number. Actual: ' + (-0 % -0));
}

//CHECK#3
if (isNaN(0 % 0) !== true) {
  throw new Test262Error('#3: 0 % 0 === Not-a-Number. Actual: ' + (0 % 0));
}

//CHECK#4
if (isNaN(0 % -0) !== true) {
  throw new Test262Error('#4: 0 % -0 === Not-a-Number. Actual: ' + (0 % -0));
}

//CHECK#5
if (isNaN(-1 % 0) !== true) {
  throw new Test262Error('#5: 1 % 0 === Not-a-Number. Actual: ' + (1 % 0));
}

//CHECK#6
if (isNaN(-1 % -0) !== true) {
  throw new Test262Error('#6: -1 % -0 === Not-a-Number. Actual: ' + (-1 % -0));
}

//CHECK#7
if (isNaN(1 % 0) !== true) {
  throw new Test262Error('#7: 1 % 0 === Not-a-Number. Actual: ' + (1 % 0));
}

//CHECK#8
if (isNaN(1 % -0) !== true) {
  throw new Test262Error('#8: 1 % -0 === Not-a-Number. Actual: ' + (1 % -0));
}

//CHECK#9
if (isNaN(Number.NEGATIVE_INFINITY % 0) !== true) {
  throw new Test262Error('#9: Infinity % 0 === Not-a-Number. Actual: ' + (Infinity % 0));
}

//CHECK#10
if (isNaN(Number.NEGATIVE_INFINITY % -0) !== true) {
  throw new Test262Error('#10: -Infinity % -0 === Not-a-Number. Actual: ' + (-Infinity % -0));
}

//CHECK#11
if (isNaN(Number.POSITIVE_INFINITY % 0) !== true) {
  throw new Test262Error('#11: Infinity % 0 === Not-a-Number. Actual: ' + (Infinity % 0));
}

//CHECK#12
if (isNaN(Number.POSITIVE_INFINITY % -0) !== true) {
  throw new Test262Error('#12: Infinity % -0 === Not-a-Number. Actual: ' + (Infinity % -0));
}

//CHECK#13
if (isNaN(Number.MIN_VALUE % 0) !== true) {
  throw new Test262Error('#13: Number.MIN_VALUE % 0 === Not-a-Number. Actual: ' + (Number.MIN_VALUE % 0));
}

//CHECK#14
if (isNaN(Number.MIN_VALUE % -0) !== true) {
  throw new Test262Error('#14: -Number.MIN_VALUE % -0 === Not-a-Number. Actual: ' + (-Number.MIN_VALUE % -0));
}

//CHECK#15
if (isNaN(Number.MAX_VALUE % 0) !== true) {
  throw new Test262Error('#15: Number.MAX_VALUE % 0 === Not-a-Number. Actual: ' + (Number.MAX_VALUE % 0));
}

//CHECK#16
if (isNaN(Number.MAX_VALUE % -0) !== true) {
  throw new Test262Error('#16: Number.MAX_VALUE % -0 === Not-a-Number. Actual: ' + (Number.MAX_VALUE % -0));
}
