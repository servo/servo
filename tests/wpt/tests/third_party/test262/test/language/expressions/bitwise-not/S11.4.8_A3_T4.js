// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator ~x returns ~ToInt32(x)
es5id: 11.4.8_A3_T4
description: Type(x) is undefined or null
---*/

//CHECK#1
if (~void 0 !== -1) {
  throw new Test262Error('#1: ~void 0 === -1. Actual: ' + (~void 0));
}

//CHECK#2
if (~null !== -1) {
  throw new Test262Error('#2: ~null === -1. Actual: ' + (~null));
}
