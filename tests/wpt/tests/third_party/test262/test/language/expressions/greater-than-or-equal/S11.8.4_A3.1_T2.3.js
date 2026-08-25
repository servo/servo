// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is not String or Type(Primitive(y)) is not String,
    then operator x >= y returns ToNumber(x) >= ToNumber(y)
es5id: 11.8.4_A3.1_T2.3
description: >
    Type(Primitive(x)) is different from Type(Primitive(y)) and both
    types vary between Number (primitive or object) and Null
---*/

//CHECK#1
if (1 >= null !== true) {
  throw new Test262Error('#1: 1 >= null === true');
}

//CHECK#2
if (null >= 1 !== false) {
  throw new Test262Error('#2: null >= 1 === false');
}

//CHECK#3
if (new Number(1) >= null !== true) {
  throw new Test262Error('#3: new Number(1) >= null === true');
}

//CHECK#4
if (null >= new Number(1) !== false) {
  throw new Test262Error('#4: null >= new Number(1) === false');
}
