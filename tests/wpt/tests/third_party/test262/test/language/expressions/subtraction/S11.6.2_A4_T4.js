// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x - y produces the same result as x + (-y)
es5id: 11.6.2_A4_T4
description: >
    The difference of an infinity and a finite value is equal to
    infinity of appropriate sign
---*/

//CHECK#1
if (Number.POSITIVE_INFINITY - 1 !== Number.POSITIVE_INFINITY ) {
  throw new Test262Error('#1: Infinity - 1 === Infinity. Actual: ' + (Infinity - 1));
}

//CHECK#2
if (-1 - Number.POSITIVE_INFINITY !== Number.NEGATIVE_INFINITY ) {
  throw new Test262Error('#2: -1 - Infinity === -Infinity. Actual: ' + (-1 - Infinity));
}

//CHECK#3
if (Number.NEGATIVE_INFINITY - 1 !== Number.NEGATIVE_INFINITY ) {
  throw new Test262Error('#3: -Infinity - 1 === -Infinity. Actual: ' + (-Infinity - 1));
}

//CHECK#4
if (-1 - Number.NEGATIVE_INFINITY !== Number.POSITIVE_INFINITY ) {
  throw new Test262Error('#4: -1 - -Infinity === Infinity. Actual: ' + (-1 - -Infinity));
}

//CHECK#5
if (Number.POSITIVE_INFINITY - Number.MAX_VALUE !== Number.POSITIVE_INFINITY ) {
  throw new Test262Error('#5: Infinity - Number.MAX_VALUE === Infinity. Actual: ' + (Infinity - Number.MAX_VALUE));
}

//CHECK#6
if (-Number.MAX_VALUE - Number.POSITIVE_INFINITY !== Number.NEGATIVE_INFINITY ) {
  throw new Test262Error('#6: -Number.MAX_VALUE - Infinity === I-nfinity. Actual: ' + (-Number.MAX_VALUE - Infinity));
}

//CHECK#7
if (Number.NEGATIVE_INFINITY - Number.MAX_VALUE !== Number.NEGATIVE_INFINITY ) {
  throw new Test262Error('#7: -Infinity - Number.MAX_VALUE === -Infinity. Actual: ' + (-Infinity - Number.MAX_VALUE));
}

//CHECK#8
if (-Number.MAX_VALUE - Number.NEGATIVE_INFINITY !== Number.POSITIVE_INFINITY ) {
  throw new Test262Error('#8: -Number.MAX_VALUE - -Infinity === Infinity. Actual: ' + (-Number.MAX_VALUE - -Infinity));
}
