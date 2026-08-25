// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator -x returns -ToNumber(x)
es5id: 11.4.7_A3_T2
description: Type(x) is number primitive or Number object
---*/

//CHECK#1
if (-(1) !== -1) {
  throw new Test262Error('#1: -(1) === -1. Actual: ' + (-(1)));
}

//CHECK#2
if (-new Number(-1) !== 1) {
  throw new Test262Error('#2: -new Number(-1) === 1. Actual: ' + (-new Number(-1)));
}
