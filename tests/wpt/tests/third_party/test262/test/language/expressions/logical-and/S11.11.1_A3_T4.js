// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is false, return x
es5id: 11.11.1_A3_T4
description: Type(x) or Type(y) is changed between null and undefined
---*/

//CHECK#1
if ((undefined && true) !== undefined) {
  throw new Test262Error('#1: (undefined && true) === undefined');
}

//CHECK#2
if ((null && false) !== null) {
  throw new Test262Error('#2: (null && false) === null');
}
