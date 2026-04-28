// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToNumber(value) is NaN, ToInteger(value) returns +0
es5id: 9.4_A1
description: >
    Check what position is defined by Number.NaN in string "abc":
    "abc".charAt(Number.NaN)
---*/

// CHECK#1
if ("abc".charAt(Number.NaN) !== "a") {
  throw new Test262Error('#1: "abc".charAt(Number.NaN) === "a". Actual: ' + ("abc".charAt(Number.NaN)));
}

// CHECK#2
if ("abc".charAt("x") !== "a") {
  throw new Test262Error('#2: "abc".charAt("x") === "a". Actual: ' + ("abc".charAt("x")));
}
