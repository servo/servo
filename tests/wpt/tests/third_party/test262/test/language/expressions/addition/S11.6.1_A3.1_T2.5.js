// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is not String and Type(Primitive(y)) is not String,
    then operator x + y returns ToNumber(x) + ToNumber(y)
es5id: 11.6.1_A3.1_T2.5
description: >
    Type(Primitive(x)) is different from Type(Primitive(y)) and both
    types vary between Boolean (primitive or object) and Null
---*/

//CHECK#1
if (true + null !== 1) {
  throw new Test262Error('#1: true + null === 1. Actual: ' + (true + null));
}

//CHECK#2
if (null + true !== 1) {
  throw new Test262Error('#2: null + true === 1. Actual: ' + (null + true));
}

//CHECK#3
if (new Boolean(true) + null !== 1) {
  throw new Test262Error('#3: new Boolean(true) + null === 1. Actual: ' + (new Boolean(true) + null));
}

//CHECK#4
if (null + new Boolean(true) !== 1) {
  throw new Test262Error('#4: null + new Boolean(true) === 1. Actual: ' + (null + new Boolean(true)));
}
