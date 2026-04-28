// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x | y returns ToNumber(x) | ToNumber(y)
es5id: 11.10.3_A3_T2.8
description: >
    Type(x) is different from Type(y) and both types vary between
    Boolean (primitive or object) and Undefined
---*/

//CHECK#1
if ((true | undefined) !== 1) {
  throw new Test262Error('#1: (true | undefined) === 1. Actual: ' + ((true | undefined)));
}

//CHECK#2
if ((undefined | true) !== 1) {
  throw new Test262Error('#2: (undefined | true) === 1. Actual: ' + ((undefined | true)));
}

//CHECK#3
if ((new Boolean(true) | undefined) !== 1) {
  throw new Test262Error('#3: (new Boolean(true) | undefined) === 1. Actual: ' + ((new Boolean(true) | undefined)));
}

//CHECK#4
if ((undefined | new Boolean(true)) !== 1) {
  throw new Test262Error('#4: (undefined | new Boolean(true)) === 1. Actual: ' + ((undefined | new Boolean(true))));
}
