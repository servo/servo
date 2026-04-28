// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return y
es5id: 11.12_A4_T4
description: Type(x) or Type(y) is changed between null and undefined
---*/

//CHECK#1
if ((true ? undefined : true) !== undefined) {
  throw new Test262Error('#1: (true ? undefined : true) === undefined');
}

//CHECK#2
if ((true ? null : true) !== null) {
  throw new Test262Error('#2: (true ? null : true) === null');
}
