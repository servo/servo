// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalEscape :: DecimalIntegerLiteral [lookahead not in DecimalDigit]"
es5id: 15.10.2.11_A1_T4
description: DecimalIntegerLiteral is not 0
---*/

var arr = /(A)\1/.exec("AA");

if ((arr === null) || (arr[0] !== "AA")) {
  throw new Test262Error('#1: var arr = (/(A)\\1/.exec("AA")); arr[0] === "AA". Actual. ' + (arr && arr[0]));
}

if ((arr === null) || (arr[1] !== "A")) {
  throw new Test262Error('#2: var arr = (/(A)\\1/.exec("AA")); arr[1] === "A". Actual. ' + (arr && arr[1]));
}
