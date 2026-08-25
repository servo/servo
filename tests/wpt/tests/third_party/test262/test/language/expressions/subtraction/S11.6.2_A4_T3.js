// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x - y produces the same result as x + (-y)
es5id: 11.6.2_A4_T3
description: The difference of two infinities of the same sign is NaN
---*/

//CHECK#1
if (isNaN(Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY) !== true ) {
  throw new Test262Error('#1: Infinity - Infinity === Not-a-Number. Actual: ' + (Infinity - Infinity));
}

//CHECK#2
if (isNaN(Number.NEGATIVE_INFINITY - Number.NEGATIVE_INFINITY) !== true ) {
  throw new Test262Error('#2: -Infinity - -Infinity === Not-a-Number. Actual: ' + (-Infinity - -Infinity));
}
