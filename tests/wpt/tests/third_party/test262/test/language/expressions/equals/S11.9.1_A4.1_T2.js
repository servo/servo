// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x or y is NaN, return false
es5id: 11.9.1_A4.1_T2
description: y is NaN
---*/

//CHECK#1
if ((true == Number.NaN) !== false) {
  throw new Test262Error('#1: (true == NaN) === false');
}

//CHECK#2
if ((-1 == Number.NaN) !== false) {
  throw new Test262Error('#2: (-1 == NaN) === false');
}

//CHECK#3
if ((Number.NaN == Number.NaN) !== false) {
  throw new Test262Error('#3: (NaN == NaN) === false');
}

//CHECK#4
if ((Number.POSITIVE_INFINITY == Number.NaN) !== false) {
  throw new Test262Error('#4: (+Infinity == NaN) === false');
}

//CHECK#5
if ((Number.NEGATIVE_INFINITY == Number.NaN) !== false) {
  throw new Test262Error('#5: (-Infinity == NaN) === false');
}

//CHECK#6
if ((Number.MAX_VALUE == Number.NaN) !== false) {
  throw new Test262Error('#6: (Number.MAX_VALUE == NaN) === false');
}

//CHECK#7
if ((Number.MIN_VALUE == Number.NaN) !== false) {
  throw new Test262Error('#7: (Number.MIN_VALUE == NaN) === false');
}

//CHECK#8
if (("string" == Number.NaN) !== false) {
  throw new Test262Error('#8: ("string" == NaN) === false');
}

//CHECK#9
if ((new Object() == Number.NaN) !== false) {
  throw new Test262Error('#9: (new Object() == NaN) === false');
}
