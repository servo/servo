// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is String or Type(Primitive(y)) is String, then
    operator x + y returns the result of concatenating ToString(x) followed
    by ToString(y)
es5id: 11.6.1_A3.2_T2.1
description: >
    Type(Primitive(x)) is different from Type(Primitive(y)) and both
    types vary between Number (primitive or object) and String
    (primitive and object)
---*/

//CHECK#1
if ("1" + 1 !== "11") {
  throw new Test262Error('#1: "1" + 1 === "11". Actual: ' + ("1" + 1));
}

//CHECK#2
if (1 + "1" !== "11") {
  throw new Test262Error('#2: 1 + "1" === "11". Actual: ' + (1 + "1"));
}

//CHECK#3
if (new String("1") + 1 !== "11") {
  throw new Test262Error('#3: new String("1") + 1 === "11". Actual: ' + (new String("1") + 1));
}

//CHECK#4
if (1 + new String("1") !== "11") {
  throw new Test262Error('#4: 1 + new String("1") === "11". Actual: ' + (1 + new String("1")));
}

//CHECK#5
if ("1" + new Number(1) !== "11") {
  throw new Test262Error('#5: "1" + new Number(1) === "11". Actual: ' + ("1" + new Number(1)));
}

//CHECK#6
if (new Number(1) + "1" !== "11") {
  throw new Test262Error('#6: new Number(1) + "1" === "11". Actual: ' + (new Number(1) + "1"));
}

//CHECK#7
if (new String("1") + new Number(1) !== "11") {
  throw new Test262Error('#7: new String("1") + new Number(1) === "11". Actual: ' + (new String("1") + new Number(1)));
}

//CHECK#8
if (new Number(1) + new String("1") !== "11") {
  throw new Test262Error('#8: new Number(1) + new String("1") === "11". Actual: ' + (new Number(1) + new String("1")));
}

//CHECK#9
if ("x" + 1 !=="x1") {
  throw new Test262Error('#9: "x" + 1 === "x1". Actual: ' + ("x" + 1));
}

//CHECK#10
if (1 + "x" !== "1x") {
  throw new Test262Error('#10: 1 + "x" === "1x". Actual: ' + (1 + "x"));
}
