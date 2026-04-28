// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalEscape :: DecimalIntegerLiteral [lookahead not in DecimalDigit]"
es5id: 15.10.2.11_A1_T5
description: DecimalIntegerLiteral is not 0
---*/

var arr = /\1(A)/.exec("AA");

if ((arr === null) || (arr[0] !== "A")) {
  throw new Test262Error('#1: var arr = (/\\1(A)/.exec("AA")); arr[0] === "A". Actual. ' + (arr && arr[0]));
}

if ((arr === null) || (arr[1] !== "A")) {
  throw new Test262Error('#2: var arr = (/\\1(A)/.exec("AA")); arr[1] === "A". Actual. ' + (arr && arr[1]));
}
