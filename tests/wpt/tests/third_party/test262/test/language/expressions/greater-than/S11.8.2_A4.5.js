// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is +Infinity and x !== y, return true
es5id: 11.8.2_A4.5
description: y is number primitive
---*/

//CHECK#1
if ((Number.POSITIVE_INFINITY > 0) !== true) {
  throw new Test262Error('#1: (+Infinity > 0) === true');
}

//CHECK#2
if ((Number.POSITIVE_INFINITY > 1.1) !== true) {
  throw new Test262Error('#2: (+Infinity > 1.1) === true');
}

//CHECK#3
if ((Number.POSITIVE_INFINITY > -1.1) !== true) {
  throw new Test262Error('#3: (+Infinity > -1.1) === true');
}

//CHECK#4
if ((Number.POSITIVE_INFINITY > Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#4: (+Infinity > -Infinity) === true');
}

//CHECK#5
if ((Number.POSITIVE_INFINITY > Number.MAX_VALUE) !== true) {
  throw new Test262Error('#5: (+Infinity > Number.MAX_VALUE) === true');
}

//CHECK#6
if ((Number.POSITIVE_INFINITY > Number.MIN_VALUE) !== true) {
  throw new Test262Error('#6: (+Infinity > Number.MIN_VALUE) === true');
}
