// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of division is determined by the specification of IEEE 754
    arithmetics
es5id: 11.5.2_A4_T6
description: >
    Division of a finite value by an infinity results in zero of
    appropriate sign
---*/

//CHECK#1
if (1 / Number.NEGATIVE_INFINITY !== -0) {
  throw new Test262Error('#1.1: 1 / -Infinity === 0. Actual: ' + (1 / -Infinity));
} else {
  if (1 / (1 / Number.NEGATIVE_INFINITY) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#1.2: 1 / -Infinity === - 0. Actual: +0');
  }
}

//CHECK#2
if (-1 / Number.NEGATIVE_INFINITY !== +0) {
  throw new Test262Error('#2.1: -1 / -Infinity === 0. Actual: ' + (-1 / -Infinity));
} else {
  if (1 / (-1 / Number.NEGATIVE_INFINITY) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#2.2: -1 / -Infinity === + 0. Actual: -0');
  }
}

//CHECK#3
if (1 / Number.POSITIVE_INFINITY !== +0) {
  throw new Test262Error('#3.1: 1 / Infinity === 0. Actual: ' + (1 / Infinity));
} else {
  if (1 / (1 / Number.POSITIVE_INFINITY) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#3.2: 1 / Infinity === + 0. Actual: -0');
  }
}

//CHECK#4
if (-1 / Number.POSITIVE_INFINITY !== -0) {
  throw new Test262Error('#4.1: -1 / Infinity === 0. Actual: ' + (-1 / Infinity));
} else {
  if (1 / (-1 / Number.POSITIVE_INFINITY) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#4.2: -1 / Infinity === - 0. Actual: +0');
  }
}
