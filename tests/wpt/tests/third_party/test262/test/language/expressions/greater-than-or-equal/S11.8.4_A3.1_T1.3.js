// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is not String or Type(Primitive(y)) is not String,
    then operator x >= y returns ToNumber(x) >= ToNumber(y)
es5id: 11.8.4_A3.1_T1.3
description: >
    Type(Primitive(x)) and Type(Primitive(y)) vary between Null and
    Undefined
---*/

//CHECK#1
if (null >= undefined !== false) {
  throw new Test262Error('#1: null >= undefined === false');
}

//CHECK#2
if (undefined >= null !== false) {
  throw new Test262Error('#2: undefined >= null === false');
}

//CHECK#3
if (undefined >= undefined !== false) {
  throw new Test262Error('#3: undefined >= undefined === false');
}

//CHECK#4
if (null >= null !== true) {
  throw new Test262Error('#4: null >= null === true');
}
