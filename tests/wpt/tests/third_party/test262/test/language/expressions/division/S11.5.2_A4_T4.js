// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of division is determined by the specification of IEEE 754
    arithmetics
es5id: 11.5.2_A4_T4
description: Division of an infinity by an infinity results in NaN
---*/

//CHECK#1
if (isNaN(Number.NEGATIVE_INFINITY / Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#1: -Infinity / -Infinity === Not-a-Number. Actual: ' + (-Infinity / -Infinity));
}

//CHECK#2
if (isNaN(Number.POSITIVE_INFINITY / Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#2: Infinity / Infinity === Not-a-Number. Actual: ' + (Infinity / Infinity));
}

//CHECK#3
if (isNaN(Number.NEGATIVE_INFINITY / Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#3: -Infinity / Infinity === Not-a-Number. Actual: ' + (-Infinity / Infinity));
}

//CHECK#4
if (isNaN(Number.POSITIVE_INFINITY / Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#4: Infinity / -Infinity === Not-a-Number. Actual: ' + (Infinity / -Infinity));
}
