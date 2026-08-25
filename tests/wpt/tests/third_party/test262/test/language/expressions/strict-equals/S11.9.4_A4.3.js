// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are Number-s minus NaN, +0, -0.
    Return true, if x is the same number value as y; otherwise, return false
es5id: 11.9.4_A4.3
description: x and y are primitive numbers
---*/

//CHECK#1
if (!(Number.POSITIVE_INFINITY === Number.POSITIVE_INFINITY)) {
  throw new Test262Error('#1: +Infinity === +Infinity');
}

//CHECK#2
if (!(Number.NEGATIVE_INFINITY === Number.NEGATIVE_INFINITY)) {
  throw new Test262Error('#2: -Infinity === -Infinity');
}

//CHECK#3
if (!(13 === 13)) {
  throw new Test262Error('#3: 13 === 13');
}

//CHECK#4
if (!(-13 === -13)) {
  throw new Test262Error('#4: -13 === -13');
}

//CHECK#5
if (!(1.3 === 1.3)) {
  throw new Test262Error('#5: 1.3 === 1.3');
}

//CHECK#6
if (!(-1.3 === -1.3)) {
  throw new Test262Error('#6: -1.3 === -1.3');
}

//CHECK#7
if (!(Number.POSITIVE_INFINITY === -Number.NEGATIVE_INFINITY)) {
  throw new Test262Error('#7: +Infinity === -(-Infinity)');
}

//CHECK#8
if (1 === 0.999999999999) {
  throw new Test262Error('#8: 1 !== 0.999999999999');
}

//CHECK#9
if (!(1.0 === 1)) {
  throw new Test262Error('#9: 1.0 === 1');
}
