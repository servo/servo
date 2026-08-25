// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is -Infinity, return false
es5id: 11.8.2_A4.7
description: y is number primitive
---*/

//CHECK#1
if ((Number.NEGATIVE_INFINITY > 0) !== false) {
  throw new Test262Error('#1: (-Infinity > 0) === false');
}

//CHECK#2
if ((Number.NEGATIVE_INFINITY > 1.1) !== false) {
  throw new Test262Error('#2: (-Infinity > 1.1) === false');
}

//CHECK#3
if ((Number.NEGATIVE_INFINITY > -1.1) !== false) {
  throw new Test262Error('#3: (-Infinity > -1.1) === false');
}

//CHECK#4
if ((Number.NEGATIVE_INFINITY > Number.POSITIVE_INFINITY) !== false) {
  throw new Test262Error('#4: (-Infinity > +Infinity) === false');
}

//CHECK#5
if ((Number.NEGATIVE_INFINITY > Number.MAX_VALUE) !== false) {
  throw new Test262Error('#5: (-Infinity > Number.MAX_VALUE) === false');
}

//CHECK#6
if ((Number.NEGATIVE_INFINITY > Number.MIN_VALUE) !== false) {
  throw new Test262Error('#6: (-Infinity > Number.MIN_VALUE) === false');
}
