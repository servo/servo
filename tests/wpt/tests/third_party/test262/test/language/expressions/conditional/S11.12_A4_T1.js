// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return y
es5id: 11.12_A4_T1
description: Type(y) and Type(z) are boolean primitives
---*/

//CHECK#1
if ((true ? false : true) !== false) {
  throw new Test262Error('#1: (true ? false : true) === false');
}

//CHECK#2
var y = new Boolean(true);
if ((true ? y : false) !== y) {
  throw new Test262Error('#2: (var y = new Boolean(true); (true ? y : false) === y');
}

//CHECK#3
var y = new Boolean(false);
if ((y ? y : true) !== y) {
  throw new Test262Error('#3: (var y = new Boolean(false); (y ? y : true) === y');
}
