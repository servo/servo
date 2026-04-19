// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is not String and Type(Primitive(y)) is not String,
    then operator x + y returns ToNumber(x) + ToNumber(y)
es5id: 11.6.1_A3.1_T2.1
description: >
    Type(Primitive(x)) is different from Type(Primitive(y)) and both
    types vary between Number (primitive or object) or Boolean
    (primitive and object)
---*/

//CHECK#1
if (true + 1 !== 2) {
  throw new Test262Error('#1: true + 1 === 2. Actual: ' + (true + 1));
}

//CHECK#2
if (1 + true !== 2) {
  throw new Test262Error('#2: 1 + true === 2. Actual: ' + (1 + true));
}

//CHECK#3
if (new Boolean(true) + 1 !== 2) {
  throw new Test262Error('#3: new Boolean(true) + 1 === 2. Actual: ' + (new Boolean(true) + 1));
}

//CHECK#4
if (1 + new Boolean(true) !== 2) {
  throw new Test262Error('#4: 1 + new Boolean(true) === 2. Actual: ' + (1 + new Boolean(true)));
}

//CHECK#5
if (true + new Number(1) !== 2) {
  throw new Test262Error('#5: true + new Number(1) === 2. Actual: ' + (true + new Number(1)));
}

//CHECK#6
if (new Number(1) + true !== 2) {
  throw new Test262Error('#6: new Number(1) + true === 2. Actual: ' + (new Number(1) + true));
}

//CHECK#7
if (new Boolean(true) + new Number(1) !== 2) {
  throw new Test262Error('#7: new Boolean(true) + new Number(1) === 2. Actual: ' + (new Boolean(true) + new Number(1)));
}

//CHECK#8
if (new Number(1) + new Boolean(true) !== 2) {
  throw new Test262Error('#8: new Number(1) + new Boolean(true) === 2. Actual: ' + (new Number(1) + new Boolean(true)));
}
