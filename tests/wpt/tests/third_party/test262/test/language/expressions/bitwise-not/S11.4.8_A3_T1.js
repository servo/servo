// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator ~x returns ~ToInt32(x)
es5id: 11.4.8_A3_T1
description: Type(x) is boolean primitive or Boolean object
---*/

//CHECK#1
if (~false !== -1) {
  throw new Test262Error('#1: ~false === -1. Actual: ' + (~false));
}

//CHECK#2
if (~new Boolean(true) !== -2) {
  throw new Test262Error('#2: ~new Boolean(true) === -2. Actual: ' + (~new Boolean(true)));
}

//CHECK#3
if (~new Boolean(false) !== -1) {
  throw new Test262Error('#3: ~new Boolean(false) === -1. Actual: ' + (~new Boolean(false)));
}
