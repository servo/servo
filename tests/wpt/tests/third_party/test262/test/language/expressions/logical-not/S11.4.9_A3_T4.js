// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator !x returns !ToBoolean(x)
es5id: 11.4.9_A3_T4
description: Type(x) is undefined or null
---*/

//CHECK#1
if (!void 0 !== true) {
  throw new Test262Error('#1: !void 0 === true');
}

//CHECK#2
if (!null !== true) {
  throw new Test262Error('#2: !null === true');
}
