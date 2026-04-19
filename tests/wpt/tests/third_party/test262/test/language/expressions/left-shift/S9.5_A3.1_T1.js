// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator uses ToNumber
es5id: 9.5_A3.1_T1
description: Type(x) is Boolean
---*/

// CHECK#1
if ((new Boolean(true) << 0) !== 1) {
  throw new Test262Error('#1: (new Boolean(true) << 0) === 1. Actual: ' + ((new Boolean(true) << 0)));
}

// CHECK#2
if ((false << 0) !== 0) {
  throw new Test262Error('#2: (false << 0) === 0. Actual: ' + ((false << 0)));
}
