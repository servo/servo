// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of division is determined by the specification of IEEE 754
    arithmetics
es5id: 11.5.2_A4_T5
description: >
    Division of an infinity by a finite non-zero value results in a
    signed infinity
---*/

//CHECK#1
if (Number.NEGATIVE_INFINITY / 1 !== Number.NEGATIVE_INFINITY) {
  throw new Test262Error('#1: -Infinity / 1 === -Infinity. Actual: ' + (-Infinity / 1));
}

//CHECK#2
if (Number.NEGATIVE_INFINITY / -1 !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#2: -Infinity / -1 === Infinity. Actual: ' + (-Infinity / -1));
}

//CHECK#3
if (Number.POSITIVE_INFINITY / 1 !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#3: Infinity / 1 === Infinity. Actual: ' + (Infinity / 1));
}

//CHECK#4
if (Number.POSITIVE_INFINITY / -1 !== Number.NEGATIVE_INFINITY) {
  throw new Test262Error('#4: Infinity / -1 === -Infinity. Actual: ' + (Infinity / -1));
}

//CHECK#5
if (Number.POSITIVE_INFINITY / -Number.MAX_VALUE !== Number.NEGATIVE_INFINITY) {
  throw new Test262Error('#5: Infinity / -Number.MAX_VALUE === -Infinity. Actual: ' + (Infinity / -Number.MAX_VALUE));
}

//CHECK#6
if (Number.NEGATIVE_INFINITY / Number.MIN_VALUE !== Number.NEGATIVE_INFINITY) {
  throw new Test262Error('#6: -Infinity / Number.MIN_VALUE === -Infinity. Actual: ' + (-Infinity / Number.MIN_VALUE));
}
