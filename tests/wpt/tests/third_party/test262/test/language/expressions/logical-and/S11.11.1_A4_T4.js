// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return y
es5id: 11.11.1_A4_T4
description: Type(x) or Type(y) is changed between null and undefined
---*/

//CHECK#1
if ((true && undefined) !== undefined) {
  throw new Test262Error('#1: (true && undefined) === undefined');
}

//CHECK#2
if ((true && null) !== null) {
  throw new Test262Error('#2: (true && null) === null');
}
