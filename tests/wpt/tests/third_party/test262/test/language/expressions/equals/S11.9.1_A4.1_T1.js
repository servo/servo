// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x or y is NaN, return false
es5id: 11.9.1_A4.1_T1
description: x is NaN
---*/

//CHECK#1
if ((Number.NaN == true) !== false) {
  throw new Test262Error('#1: (NaN == true) === false');
}

//CHECK#2
if ((Number.NaN == 1) !== false) {
  throw new Test262Error('#2: (NaN == 1) === false');
}

//CHECK#3
if ((Number.NaN == Number.NaN) !== false) {
  throw new Test262Error('#3: (NaN == NaN) === false');
}

//CHECK#4
if ((Number.NaN == Number.POSITIVE_INFINITY) !== false) {
  throw new Test262Error('#4: (NaN == +Infinity) === false');
}

//CHECK#5
if ((Number.NaN == Number.NEGATIVE_INFINITY) !== false) {
  throw new Test262Error('#5: (NaN == -Infinity) === false');
}

//CHECK#6
if ((Number.NaN == Number.MAX_VALUE) !== false) {
  throw new Test262Error('#6: (NaN == Number.MAX_VALUE) === false');
}

//CHECK#7
if ((Number.NaN == Number.MIN_VALUE) !== false) {
  throw new Test262Error('#7: (NaN == Number.MIN_VALUE) === false');
}

//CHECK#8
if ((Number.NaN == "string") !== false) {
  throw new Test262Error('#8: (NaN == "string") === false');
}

//CHECK#9
if ((Number.NaN == new Object()) !== false) {
  throw new Test262Error('#9: (NaN == new Object()) === false');
}
