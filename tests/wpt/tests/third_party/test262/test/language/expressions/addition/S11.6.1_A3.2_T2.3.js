// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is String or Type(Primitive(y)) is String, then
    operator x + y returns the result of concatenating ToString(x) followed
    by ToString(y)
es5id: 11.6.1_A3.2_T2.3
description: >
    Type(Primitive(x)) is different from Type(Primitive(y)) and both
    types vary between String (primitive or object) and Undefined
---*/

//CHECK#1
if ("1" + undefined !== "1undefined") {
  throw new Test262Error('#1: "1" + undefined === "1undefined". Actual: ' + ("1" + undefined));
}

//CHECK#2
if (undefined + "1" !== "undefined1") {
  throw new Test262Error('#2: undefined + "1" === "undefined1". Actual: ' + (undefined + "1"));
}

//CHECK#3
if (new String("1") + undefined !== "1undefined") {
  throw new Test262Error('#3: new String("1") + undefined === "1undefined". Actual: ' + (new String("1") + undefined));
}

//CHECK#4
if (undefined + new String("1") !== "undefined1") {
  throw new Test262Error('#4: undefined + new String("1") === "undefined1". Actual: ' + (undefined + new String("1")));
}
