// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x % y returns ToNumber(x) % ToNumber(y)
es5id: 11.5.3_A3_T2.3
description: >
    Type(x) is different from Type(y) and both types vary between
    Number (primitive or object) and Null
---*/

//CHECK#1
if (isNaN(1 % null) !== true) {
  throw new Test262Error('#1: 1 % null === Not-a-Number. Actual: ' + (1 % null));
}

//CHECK#2
if (null % 1 !== 0) {
  throw new Test262Error('#2: null % 1 === 0. Actual: ' + (null % 1));
}

//CHECK#3
if (isNaN(new Number(1) % null) !== true) {
  throw new Test262Error('#3: new Number(1) % null === Not-a-Number. Actual: ' + (new Number(1) % null));
}

//CHECK#4
if (null % new Number(1) !== 0) {
  throw new Test262Error('#4: null % new Number(1) === 0. Actual: ' + (null % new Number(1)));
}
