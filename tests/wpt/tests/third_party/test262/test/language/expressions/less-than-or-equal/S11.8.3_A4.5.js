// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is +Infinity and x !== y, return false
es5id: 11.8.3_A4.5
description: y is number primitive
---*/

//CHECK#1
if ((Number.POSITIVE_INFINITY <= 0) !== false) {
  throw new Test262Error('#1: (+Infinity <= 0) === false');
}

//CHECK#2
if ((Number.POSITIVE_INFINITY <= 1.1) !== false) {
  throw new Test262Error('#2: (+Infinity <= 1.1) === false');
}

//CHECK#3
if ((Number.POSITIVE_INFINITY <= -1.1) !== false) {
  throw new Test262Error('#3: (+Infinity <= -1.1) === false');
}

//CHECK#4
if ((Number.POSITIVE_INFINITY <= Number.NEGATIVE_INFINITY) !== false) {
  throw new Test262Error('#4: (+Infinity <= -Infinity) === false');
}

//CHECK#5
if ((Number.POSITIVE_INFINITY <= Number.MAX_VALUE) !== false) {
  throw new Test262Error('#5: (+Infinity <= Number.MAX_VALUE) === false');
}

//CHECK#6
if ((Number.POSITIVE_INFINITY <= Number.MIN_VALUE) !== false) {
  throw new Test262Error('#6: (+Infinity <= Number.MIN_VALUE) === false');
}
