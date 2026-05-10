// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is false, return y
es5id: 11.11.2_A3_T4
description: Type(x) or Type(y) is changed between null and undefined
---*/

//CHECK#1
if ((false || undefined) !== undefined) {
  throw new Test262Error('#1: (false || undefined) === undefined');
}

//CHECK#2
if ((false || null) !== null) {
  throw new Test262Error('#2: (false || null) === null');
}
