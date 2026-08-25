// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The production x >>= y is the same as x = x >> y
es5id: 11.13.2_A4.7_T1.4
description: Type(x) and Type(y) vary between Null and Undefined
---*/

var x;

//CHECK#1
x = null;
x >>= undefined;
if (x !== 0) {
  throw new Test262Error('#1: x = null; x >>= undefined; x === 0. Actual: ' + (x));
}

//CHECK#2
x = undefined;
x >>= null;
if (x !== 0) {
  throw new Test262Error('#2: x = undefined; x >>= null; x === 0. Actual: ' + (x));
}

//CHECK#3
x = undefined;
x >>= undefined;
if (x !== 0) {
  throw new Test262Error('#3: x = undefined; x >>= undefined; x === 0. Actual: ' + (x));
}

//CHECK#4
x = null;
x >>= null;
if (x !== 0) {
  throw new Test262Error('#4: x = null; x >>= null; x === 0. Actual: ' + (x));
}
