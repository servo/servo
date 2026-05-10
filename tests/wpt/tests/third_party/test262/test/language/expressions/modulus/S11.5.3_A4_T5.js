// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a ECMAScript floating-point remainder operation is
    determined by the rules of IEEE arithmetics
es5id: 11.5.3_A4_T5
description: >
    If dividend is finite and the divisor is an infinity, the result
    equals the dividend
---*/

//CHECK#1
if (1 % Number.NEGATIVE_INFINITY !== 1) {
  throw new Test262Error('#1: 1 % -Infinity === 1. Actual: ' + (1 % -Infinity));
}
//CHECK#2
if (1 % Number.POSITIVE_INFINITY !==1) {
  throw new Test262Error('#2: 1 % Infinity === 1. Actual: ' + (1 % Infinity));
}

//CHECK#3
if (-1 % Number.POSITIVE_INFINITY !== -1) {
  throw new Test262Error('#3: -1 % Infinity === -1. Actual: ' + (-1 % Infinity));
}

//CHECK#4
if (-1 % Number.NEGATIVE_INFINITY !== -1) {
  throw new Test262Error('#4: -1 % -Infinity === -1. Actual: ' + (-1 % -Infinity));
}

//CHECK#5
if (0 % Number.POSITIVE_INFINITY !== 0) {
  throw new Test262Error('#5.1: 0 % Infinity === 0. Actual: ' + (0 % Infinity));
} else {
  if (1 / (0 % Number.POSITIVE_INFINITY) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#5.2: 0 % Infinity === + 0. Actual: -0');
  }
}

//CHECK#6
if (0 % Number.NEGATIVE_INFINITY !== 0) {
  throw new Test262Error('#6.1: 0 % -Infinity === 0. Actual: ' + (0 % -Infinity));
} else {
  if (1 / (0 %  Number.NEGATIVE_INFINITY) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#6.2: 0 % -Infinity === + 0. Actual: -0');
  }
}

//CHECK#7
if (-0 % Number.POSITIVE_INFINITY !== -0) {
  throw new Test262Error('#7.1: -0 % Infinity === 0. Actual: ' + (-0 % Infinity));
} else {
  if (1 / (-0 % Number.POSITIVE_INFINITY) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#7.2: -0 % Infinity === - 0. Actual: +0');
  }
}

//CHECK#8
if (-0 %  Number.NEGATIVE_INFINITY !== -0) {
  throw new Test262Error('#8.1: -0 % -Infinity === 0. Actual: ' + (-0 % -Infinity));
} else {
  if (1 / (-0 %  Number.NEGATIVE_INFINITY) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#8.2: -0 % -Infinity === - 0. Actual: +0');
  }
}

//CHECK#9
if (Number.MAX_VALUE % Number.NEGATIVE_INFINITY !== Number.MAX_VALUE) {
  throw new Test262Error('#9: Number.MAX_VALUE % -Infinity === Number.MAX_VALUE. Actual: ' + (Number.MAX_VALUE % -Infinity));
}

//CHECK#10
if (Number.MAX_VALUE % Number.POSITIVE_INFINITY !== Number.MAX_VALUE) {
  throw new Test262Error('#10: Number.MAX_VALUE % Infinity === Number.MAX_VALUE. Actual: ' + (Number.MAX_VALUE % Infinity));
}

//CHECK#11
if (-Number.MAX_VALUE % Number.POSITIVE_INFINITY !== -Number.MAX_VALUE) {
  throw new Test262Error('#11: -Number.MAX_VALUE % Infinity === -Number.MAX_VALUE. Actual: ' + (-Number.MAX_VALUE % Infinity));
}

//CHECK#12
if (-Number.MAX_VALUE % Number.NEGATIVE_INFINITY !== -Number.MAX_VALUE) {
  throw new Test262Error('#12: -Number.MAX_VALUE % -Infinity === -Number.MAX_VALUE. Actual: ' + (-Number.MAX_VALUE % -Infinity));
}

//CHECK#13
if (Number.MIN_VALUE % Number.NEGATIVE_INFINITY !== Number.MIN_VALUE) {
  throw new Test262Error('#13: Number.MIN_VALUE % -Infinity === Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE % -Infinity));
}
//CHECK#14
if (Number.MIN_VALUE % Number.POSITIVE_INFINITY !== Number.MIN_VALUE) {
  throw new Test262Error('#14: Number.MIN_VALUE % Infinity === Number.MIN_VALUE. Actual: ' + (Number.MIN_VALUE % Infinity));
}

//CHECK#15
if (-Number.MIN_VALUE % Number.POSITIVE_INFINITY !== -Number.MIN_VALUE) {
  throw new Test262Error('#15: -Number.MIN_VALUE % Infinity === -Number.MIN_VALUE. Actual: ' + (-Number.MIN_VALUE % Infinity));
}

//CHECK#16
if (-Number.MIN_VALUE % Number.NEGATIVE_INFINITY !== -Number.MIN_VALUE) {
  throw new Test262Error('#16: -Number.MIN_VALUE % -Infinity === -Number.MIN_VALUE. Actual: ' + (-Number.MIN_VALUE % -Infinity));
}
