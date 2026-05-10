// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of ToString conversion from boolean value is "true" if
    the argument is "true", else is "false"
es5id: 9.8_A3_T1
description: True and false convert to String by explicit transformation
---*/

// CHECK#1
if (String(false) !== "false") {
  throw new Test262Error('#1: String(false) === "false". Actual: ' + (String(false)));
}

// CHECK#2
if (String(true) !== "true") {
  throw new Test262Error('#2: String(true) === "true". Actual: ' + (String(true)));
}
