// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x & y returns ToNumber(x) & ToNumber(y)
es5id: 11.10.1_A3_T1.1
description: Type(x) and Type(y) are primitive boolean and Boolean object
---*/

//CHECK#1
if ((true & true) !== 1) {
  throw new Test262Error('#1: (true & true) === 1. Actual: ' + ((true & true)));
}

//CHECK#2
if ((new Boolean(true) & true) !== 1) {
  throw new Test262Error('#2: (new Boolean(true) & true) === 1. Actual: ' + ((new Boolean(true) & true)));
}

//CHECK#3
if ((true & new Boolean(true)) !== 1) {
  throw new Test262Error('#3: (true & new Boolean(true)) === 1. Actual: ' + ((true & new Boolean(true))));
}

//CHECK#4
if ((new Boolean(true) & new Boolean(true)) !== 1) {
  throw new Test262Error('#4: (new Boolean(true) & new Boolean(true)) === 1. Actual: ' + ((new Boolean(true) & new Boolean(true))));
}
