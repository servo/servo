// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The production x |= y is the same as x = x | y
es5id: 11.13.2_A4.11_T2.9
description: >
    Type(x) is different from Type(y) and both types vary between
    Boolean (primitive or object) and Null
---*/

var x;

//CHECK#1
x = true;
x |= null;
if (x !== 1) {
  throw new Test262Error('#1: x = true; x |= null; x === 1. Actual: ' + (x));
}

//CHECK#2
x = null;
x |= true;
if (x !== 1) {
  throw new Test262Error('#2: x = null; x |= true; x === 1. Actual: ' + (x));
}

//CHECK#3
x = new Boolean(true);
x |= null;
if (x !== 1) {
  throw new Test262Error('#3: x = new Boolean(true); x |= null; x === 1. Actual: ' + (x));
}

//CHECK#4
x = null;
x |= new Boolean(true);
if (x !== 1) {
  throw new Test262Error('#4: x = null; x |= new Boolean(true); x === 1. Actual: ' + (x));
}
