// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is String or Type(Primitive(y)) is String, then
    operator x + y returns the result of concatenating ToString(x) followed
    by ToString(y)
es5id: 11.6.1_A3.2_T2.4
description: >
    Type(Primitive(x)) is different from Type(Primitive(y)) and both
    types vary between String (primitive or object) and Null
---*/

//CHECK#1
if ("1" + null !== "1null") {
  throw new Test262Error('#1: "1" + null === "1null". Actual: ' + ("1" + null));
}

//CHECK#2
if (null + "1" !== "null1") {
  throw new Test262Error('#2: null + "1" === "null1". Actual: ' + (null + "1"));
}

//CHECK#3
if (new String("1") + null !== "1null") {
  throw new Test262Error('#3: new String("1") + null === "1null". Actual: ' + (new String("1") + null));
}

//CHECK#4
if (null + new String("1") !== "null1") {
  throw new Test262Error('#4: null + new String("1") === "null1". Actual: ' + (null + new String("1")));
}
