// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Compute the longest prefix of Result(2), which might be Result(2) itself,
    which satisfies the syntax of a StrDecimalLiteral
esid: sec-parsefloat-string
description: "\"Infinity\"+\"some string\""
---*/

//CHECK#1
if (parseFloat("Infinity1") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#1: parseFloat("Infinity1") === Number.POSITIVE_INFINITY. Actual: ' + (parseFloat("Infinity1")));
}

//CHECK#2
if (parseFloat("Infinityx") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#2: parseFloat("Infinityx") === Number.POSITIVE_INFINITY. Actual: ' + (parseFloat("Infinityx")));
}

//CHECK#3
if (parseFloat("Infinity+1") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#3: parseFloat("Infinity+1") === Number.POSITIVE_INFINITY. Actual: ' + (parseFloat("Infinity+1")));
}
