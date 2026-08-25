// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x ^ y returns ToNumber(x) ^ ToNumber(y)
es5id: 11.10.2_A3_T1.4
description: Type(x) and Type(y) are null and undefined
---*/

//CHECK#1
if ((null ^ undefined) !== 0) {
  throw new Test262Error('#1: (null ^ undefined) === 0. Actual: ' + ((null ^ undefined)));
}

//CHECK#2
if ((undefined ^ null) !== 0) {
  throw new Test262Error('#2: (undefined ^ null) === 0. Actual: ' + ((undefined ^ null)));
}

//CHECK#3
if ((undefined ^ undefined) !== 0) {
  throw new Test262Error('#3: (undefined ^ undefined) === 0. Actual: ' + ((undefined ^ undefined)));
}

//CHECK#4
if ((null ^ null) !== 0) {
  throw new Test262Error('#4: (null ^ null) === 0. Actual: ' + ((null ^ null)));
}
