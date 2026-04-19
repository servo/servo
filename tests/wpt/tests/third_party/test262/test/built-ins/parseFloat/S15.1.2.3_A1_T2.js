// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-parsefloat-string
description: Checking for number primitive
---*/

//CHECK#1
if (parseFloat(-1.1) !== parseFloat("-1.1")) {
  throw new Test262Error('#1: parseFloat(-1.1) === parseFloat("-1.1"). Actual: ' + (parseFloat(-1.1)));
}

//CHECK#2
if (parseFloat(Infinity) !== parseFloat("Infinity")) {
  throw new Test262Error('#2: parseFloat(Infinity) === parseFloat("Infinity"). Actual: ' + (parseFloat(Infinity)));
}

//CHECK#3
if (String(parseFloat(NaN)) !== "NaN") {
  throw new Test262Error('#3: String(parseFloat(NaN)) === "NaN". Actual: ' + (String(parseFloat(NaN))));
}

//CHECK#4
if (parseFloat(.01e+2) !== parseFloat(".01e+2")) {
  throw new Test262Error('#4: parseFloat(.01e+2) === parseFloat(".01e+2"). Actual: ' + (parseFloat(.01e+2)));
}

//CHECK#5
if (parseFloat(-0) !== 0) {
  throw new Test262Error('#5: parseFloat(-0) === 0. Actual: ' + (parseFloat(-0)));
} else {
  if (1 / parseFloat(-0) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#5: parseFloat(-0) === +0. Actual: ' + (parseFloat(-0)));
  }
}
