// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalEscape :: DecimalIntegerLiteral [lookahead not in DecimalDigit]"
es5id: 15.10.2.11_A1_T6
description: DecimalIntegerLiteral is not 0
---*/

var arr = /(A)\1(B)\2/.exec("AABB");

if ((arr === null) || (arr[0] !== "AABB")) {
  throw new Test262Error('#1: var arr = /(A)\\1(B)\\2/.exec("AABB"); arr[0] === "AABB". Actual. ' + (arr && arr[0]));
}

if ((arr === null) || (arr[1] !== "A")) {
  throw new Test262Error('#2: var arr = /(A)\\1(B)\\2/.exec("AABB"); arr[1] === "A". Actual. ' + (arr && arr[1]));
}

if ((arr === null) || (arr[2] !== "B")) {
  throw new Test262Error('#3: var arr = /(A)\\1(B)\\2/.exec("AABB"); arr[2] === "B". Actual. ' + (arr && arr[2]));
}
