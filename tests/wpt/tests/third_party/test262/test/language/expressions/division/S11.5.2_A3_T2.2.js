// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x / y returns ToNumber(x) / ToNumber(y)
es5id: 11.5.2_A3_T2.2
description: >
    Type(x) is different from Type(y) and both types vary between
    Number (primitive or object) and String (primitive and object)
---*/

//CHECK#1
if ("1" / 1 !== 1) {
  throw new Test262Error('#1: "1" / 1 === 1. Actual: ' + ("1" / 1));
}

//CHECK#2
if (1 / "1" !== 1) {
  throw new Test262Error('#2: 1 / "1" === 1. Actual: ' + (1 / "1"));
}

//CHECK#3
if (new String("1") / 1 !== 1) {
  throw new Test262Error('#3: new String("1") / 1 === 1. Actual: ' + (new String("1") / 1));
}

//CHECK#4
if (1 / new String("1") !== 1) {
  throw new Test262Error('#4: 1 / new String("1") === 1. Actual: ' + (1 / new String("1")));
}

//CHECK#5
if ("1" / new Number(1) !== 1) {
  throw new Test262Error('#5: "1" / new Number(1) === 1. Actual: ' + ("1" / new Number(1)));
}

//CHECK#6
if (new Number(1) / "1" !== 1) {
  throw new Test262Error('#6: new Number(1) / "1" === 1. Actual: ' + (new Number(1) / "1"));
}

//CHECK#7
if (new String("1") / new Number(1) !== 1) {
  throw new Test262Error('#7: new String("1") / new Number(1) === 1. Actual: ' + (new String("1") / new Number(1)));
}

//CHECK#8
if (new Number(1) / new String("1") !== 1) {
  throw new Test262Error('#8: new Number(1) / new String("1") === 1. Actual: ' + (new Number(1) / new String("1")));
}

//CHECK#9
if (isNaN("x" / 1) !== true) {
  throw new Test262Error('#9: "x" / 1 === Not-a-Number. Actual: ' + ("x" / 1));
}

//CHECK#10
if (isNaN(1 / "x") !== true) {
  throw new Test262Error('#10: 1 / "x" === Not-a-Number. Actual: ' + (1 / "x"));
}
