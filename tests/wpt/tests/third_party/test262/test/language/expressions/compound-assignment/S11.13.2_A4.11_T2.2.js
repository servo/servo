// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The production x |= y is the same as x = x | y
es5id: 11.13.2_A4.11_T2.2
description: >
    Type(x) is different from Type(y) and both types vary between
    Number (primitive or object) and String (primitive and object)
---*/

var x;

//CHECK#1
x = "1";
x |= 1;
if (x !== 1) {
  throw new Test262Error('#1: x = "1"; x |= 1; x === 1. Actual: ' + (x));
}

//CHECK#2
x = 1;
x |= "1";
if (x !== 1) {
  throw new Test262Error('#2: x = 1; x |= "1"; x === 1. Actual: ' + (x));
}

//CHECK#3
x = new String("1");
x |= 1;
if (x !== 1) {
  throw new Test262Error('#3: x = new String("1"); x |= 1; x === 1. Actual: ' + (x));
}

//CHECK#4
x = 1;
x |= new String("1");
if (x !== 1) {
  throw new Test262Error('#4: x = 1; x |= new String("1"); x === 1. Actual: ' + (x));
}

//CHECK#5
x = "1";
x |= new Number(1);
if (x !== 1) {
  throw new Test262Error('#5: x = "1"; x |= new Number(1); x === 1. Actual: ' + (x));
}

//CHECK#6
x = new Number(1);
x |= "1";
if (x !== 1) {
  throw new Test262Error('#6: x = new Number(1); x |= "1"; x === 1. Actual: ' + (x));
}

//CHECK#7
x = new String("1");
x |= new Number(1);
if (x !== 1) {
  throw new Test262Error('#7: x = new String("1"); x |= new Number(1); x === 1. Actual: ' + (x));
}

//CHECK#8
x = new Number(1);
x |= new String("1");
if (x !== 1) {
  throw new Test262Error('#8: x = new Number(1); x |= new String("1"); x === 1. Actual: ' + (x));
}

//CHECK#9
x = "x";
x |= 1;
if (x !== 1) {
  throw new Test262Error('#9: x = "x"; x |= 1; x === 1. Actual: ' + (x));
}

//CHECK#10
x = 1;
x |= "x";
if (x !== 1) {
  throw new Test262Error('#10: x = 1; x |= "x"; x === 1. Actual: ' + (x));
}
