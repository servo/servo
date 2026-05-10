// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    MemberExpression calls ToObject(MemberExpression) and
    ToString(Expression). CallExpression calls ToObject(CallExpression) and
    ToString(Expression)
es5id: 11.2.1_A3_T2
description: Checking Number case
---*/

//CHECK#1
if (1..toString() !== "1") {
  throw new Test262Error('#1: 1..toString() === "1". Actual: ' + (1..toString()));
}

//CHECK#2
if (1.1.toFixed(5) !== "1.10000") {
  throw new Test262Error('#2: 1.1.toFixed(5) === "1.10000". Actual: ' + (1.1.toFixed(5)));
}

//CHECK#3
if (1["toString"]() !== "1") {
  throw new Test262Error('#3: 1["toString"]() === "1". Actual: ' + (1["toString"]()));
}

//CHECK#4
if (1.["toFixed"](5) !== "1.00000") {
  throw new Test262Error('#4: 1.["toFixed"](5) === "1.00000". Actual: ' + (1.["toFixed"](5)));
}

//CHECK#5
if (new Number(1).toString() !== "1") {
  throw new Test262Error('#5: new Number(1).toString() === "1". Actual: ' + (new Number(1).toString()));
}

//CHECK#6
if (new Number(1)["toFixed"](5) !== "1.00000") {
  throw new Test262Error('#6: new Number(1)["toFixed"](5) === "1.00000". Actual: ' + (new Number(1)["toFixed"](5)));
}
