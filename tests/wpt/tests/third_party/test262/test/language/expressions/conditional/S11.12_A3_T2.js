// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is false, return z
es5id: 11.12_A3_T2
description: Type(y) and Type(z) are number primitives
---*/

//CHECK#1
if ((0 ? 0 : 1) !== 1) {
  throw new Test262Error('#1: (0 ? 0 : 1) === 1');
}

//CHECK#2
var z = new Number(1);
if ((0 ? 1 : z) !== z) {
  throw new Test262Error('#2: (var y = new Number(1); (0 ? 1 : z) === z');
}
