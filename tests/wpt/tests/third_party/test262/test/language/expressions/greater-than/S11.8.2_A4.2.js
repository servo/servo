// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If y is NaN, return false (if result in 11.8.5 is undefined, return false)
es5id: 11.8.2_A4.2
description: x is number primitive
---*/

//CHECK#1
if ((0 > Number.NaN) !== false) {
  throw new Test262Error('#1: (0 > NaN) === false');
}

//CHECK#2
if ((1.1 > Number.NaN) !== false) {
  throw new Test262Error('#2: (1.1 > NaN) === false');
}

//CHECK#3
if ((-1.1 > Number.NaN) !== false) {
  throw new Test262Error('#3: (-1.1 > NaN) === false');
}

//CHECK#4
if ((Number.NaN > Number.NaN) !== false) {
  throw new Test262Error('#4: (NaN > NaN) === false');
}

//CHECK#5
if ((Number.POSITIVE_INFINITY > Number.NaN) !== false) {
  throw new Test262Error('#5: (+Infinity > NaN) === false');
}

//CHECK#6
if ((Number.NEGATIVE_INFINITY > Number.NaN) !== false) {
  throw new Test262Error('#6: (-Infinity > NaN) === false');
}

//CHECK#7
if ((Number.MAX_VALUE > Number.NaN) !== false) {
  throw new Test262Error('#7: (Number.MAX_VALUE > NaN) === false');
}

//CHECK#8
if ((Number.MIN_VALUE > Number.NaN) !== false) {
  throw new Test262Error('#8: (Number.MIN_VALUE > NaN) === false');
}
