// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of ToString conversion from boolean value is "true" if
    the argument is "true", else is "false"
es5id: 9.8_A3_T2
description: True and false convert to String by implicit transformation
---*/

// CHECK#1
if (false + "" !== "false") {
  throw new Test262Error('#1: false + "" === "false". Actual: ' + (false + ""));
}

// CHECK#2
if (true + "" !== "true") {
  throw new Test262Error('#2: true + "" === "true". Actual: ' + (true + ""));	
}
