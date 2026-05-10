// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of ToString conversion from null value is "null"
es5id: 9.8_A2_T2
description: null convert to String by implicit transformation
---*/

// CHECK#1
if (null + "" !== "null") {
  throw new Test262Error('#1: null + "" === "null". Actual: ' + (null + "")); 
}
