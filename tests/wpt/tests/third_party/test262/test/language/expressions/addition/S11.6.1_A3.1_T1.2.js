// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is not String and Type(Primitive(y)) is not String,
    then operator x + y returns ToNumber(x) + ToNumber(y)
es5id: 11.6.1_A3.1_T1.2
description: >
    Type(Primitive(x)) and Type(Primitive(y)) vary between primitive
    number and Number object
---*/

//CHECK#1
if (1 + 1 !== 2) {
  throw new Test262Error('#1: 1 + 1 === 2. Actual: ' + (1 + 1));
}

//CHECK#2
if (new Number(1) + 1 !== 2) {
  throw new Test262Error('#2: new Number(1) + 1 === 2. Actual: ' + (new Number(1) + 1));
}

//CHECK#3
if (1 + new Number(1) !== 2) {
  throw new Test262Error('#3: 1 + new Number(1) === 2. Actual: ' + (1 + new Number(1)));
}

//CHECK#4
if (new Number(1) + new Number(1) !== 2) {
  throw new Test262Error('#4: new Number(1) + new Number(1) === 2. Actual: ' + (new Number(1) + new Number(1)));
}
