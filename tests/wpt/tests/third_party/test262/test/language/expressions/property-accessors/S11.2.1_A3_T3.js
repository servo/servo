// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    MemberExpression calls ToObject(MemberExpression) and
    ToString(Expression). CallExpression calls ToObject(CallExpression) and
    ToString(Expression)
es5id: 11.2.1_A3_T3
description: Checking String case;
---*/

//CHECK#1
if ("abc123".charAt(5) !== "3") {
  throw new Test262Error('#1: "abc123".charAt(5) === "3". Actual: ' + ("abc123".charAt(5)));
}

//CHECK#2
if ("abc123"["charAt"](0) !== "a") {
  throw new Test262Error('#2: "abc123"["charAt"](0) === "a". Actual: ' + ("abc123"["charAt"](0)));
}

//CHECK#3
if ("abc123".length !== 6) {
  throw new Test262Error('#3: "abc123".length === 6. Actual: ' + ("abc123".length));
}

//CHECK#4
if ("abc123"["length"] !== 6) {
  throw new Test262Error('#4: "abc123"["length"] === 6. Actual: ' + ("abc123"["length"]));
}

//CHECK#5
if (new String("abc123").length !== 6) {
  throw new Test262Error('#5: new String("abc123").length === 6. Actual: ' + (new String("abc123").length));
}

//CHECK#6
if (new String("abc123")["charAt"](2) !== "c") {
  throw new Test262Error('#6: new String("abc123")["charAt"](2) === "c". Actual: ' + (new String("abc123")["charAt"](2)));
}
