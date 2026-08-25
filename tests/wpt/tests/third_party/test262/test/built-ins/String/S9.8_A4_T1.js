// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of String conversion from string value is the input argument (no
    conversion)
es5id: 9.8_A4_T1
description: Some strings convert to String with explicit transformation
---*/

// CHECK#1
var x1 = "abc";
if (String(x1) !== x1) {
  throw new Test262Error('#1: String("abc") === "abc". Actual: ' + (String("abc")));
}

// CHECK#2
var x2 = "abc";
if (typeof String(x2) !== typeof x2) {
  throw new Test262Error('#2: typeof String("abc") === "string". Actual: ' + (typeof String("abc")));
}
