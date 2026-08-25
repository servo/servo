// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The production x += y is the same as x = x + y
es5id: 11.13.2_A4.4_T1.2
description: Type(x) and Type(y) vary between primitive number and Number object
---*/

var x;

//CHECK#1
x = 1;
x += 1;
if (x !== 2) {
  throw new Test262Error('#1: x = 1; x += 1; x === 2. Actual: ' + (x));
}

//CHECK#2
x = new Number(1);
x += 1;
if (x !== 2) {
  throw new Test262Error('#2: x = new Number(1); x += 1; x === 2. Actual: ' + (x));
}

//CHECK#3
x = 1;
x += new Number(1);
if (x !== 2) {
  throw new Test262Error('#3: x = 1; x += new Number(1); x === 2. Actual: ' + (x));
}

//CHECK#4
x = new Number(1);
x += new Number(1);
if (x !== 2) {
  throw new Test262Error('#4: x = new Number(1); x += new Number(1); x === 2. Actual: ' + (x));
}
