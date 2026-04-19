// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Return the number value for the MV of Result(4)
esid: sec-parsefloat-string
description: Checking . DecimalDigits ExponentPart_opt
---*/

//CHECK#1
if (parseFloat("+.1") !== 0.1) {
  throw new Test262Error('#1: parseFloat("+.1") === 0.1. Actual: ' + (parseFloat("+.1")));
}

//CHECK#2
if (parseFloat(".01") !== 0.01) {
  throw new Test262Error('#2: parseFloat(".01") === 0.01. Actual: ' + (parseFloat(".01")));
}

//CHECK#3
if (parseFloat("+.22e-1") !== 0.022) {
  throw new Test262Error('#3: parseFloat("+.22e-1") === 0.022. Actual: ' + (parseFloat("+.22e-1")));
}
