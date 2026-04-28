// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If y is -Infinity and x !== y, return true
es5id: 11.8.2_A4.8
description: x is number primitive
---*/

//CHECK#1
if ((0 > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#1: (0 > -Infinity) === true');
}

//CHECK#2
if ((1.1 > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#2: (1.1 > -Infinity) === true');
}

//CHECK#3
if ((-1.1 > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#3: (-1.1 > -Infinity) === true');
}

//CHECK#4
if ((Number.POSITIVE_INFINITY > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#4: (+Infinity > -Infinity) === true');
}

//CHECK#5
if ((Number.MAX_VALUE > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#5: (Number.MAX_VALUE > -Infinity) === true');
}

//CHECK#6
if ((Number.MIN_VALUE > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#6: (Number.MIN_VALUE > -Infinity) === true');
}
