// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The production x >>>= y is the same as x = x >>> y
es5id: 11.13.2_A4.8_T1.1
description: >
    Type(x) and Type(y) vary between primitive boolean and Boolean
    object
---*/

var x;

//CHECK#1
x = true;
x >>>= true;
if (x !== 0) {
  throw new Test262Error('#1: x = true; x >>>= true; x === 0. Actual: ' + (x));
}

//CHECK#2
x = new Boolean(true);
x >>>= true;
if (x !== 0) {
  throw new Test262Error('#2: x = new Boolean(true); x >>>= true; x === 0. Actual: ' + (x));
}

//CHECK#3
x = true;
x >>>= new Boolean(true);
if (x !== 0) {
  throw new Test262Error('#3: x = true; x >>>= new Boolean(true); x === 0. Actual: ' + (x));
}

//CHECK#4
x = new Boolean(true);
x >>>= new Boolean(true);
if (x !== 0) {
  throw new Test262Error('#4: x = new Boolean(true); x >>>= new Boolean(true); x === 0. Actual: ' + (x));
}
