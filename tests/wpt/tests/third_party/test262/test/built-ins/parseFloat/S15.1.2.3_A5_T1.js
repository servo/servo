// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Return the number value for the MV of Result(4)
esid: sec-parsefloat-string
description: Checking Infinity
---*/

//CHECK#1
if (parseFloat("Infinity") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#1: parseFloat("Infinity") === Number.POSITIVE_INFINITY. Actual: ' + (parseFloat("Infinity")));
}

//CHECK#2
if (parseFloat("+Infinity") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#2: parseFloat("+Infinity") === Number.POSITIVE_INFINITY. Actual: ' + (parseFloat("+Infinity")));
}

//CHECK#3
if (parseFloat("-Infinity") !== Number.NEGATIVE_INFINITY) {
  throw new Test262Error('#3: parseFloat("-Infinity") === Number.NEGATIVE_INFINITY. Actual: ' + (parseFloat("-Infinity")));
}
