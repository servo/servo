// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalEscape :: DecimalIntegerLiteral [lookahead not in DecimalDigit]"
es5id: 15.10.2.11_A1_T7
description: DecimalIntegerLiteral is not 0
---*/

var arr = /\1(A)(B)\2/.exec("ABB");

if ((arr === null) || (arr[0] !== "ABB")) {
  throw new Test262Error('#1: var arr = /\\1(A)(B)\\2/.exec("ABB"); arr[0] === "ABB". Actual. ' + (arr && arr[0]));
}

if ((arr === null) || (arr[1] !== "A")) {
  throw new Test262Error('#2: var arr = /\\1(A)(B)\\2/.exec("ABB"); arr[1] === "A". Actual. ' + (arr && arr[1]));
}

if ((arr === null) || (arr[2] !== "B")) {
  throw new Test262Error('#3: var arr = /\\1(A)(B)\\2/.exec("ABB"); arr[2] === "B". Actual. ' + (arr && arr[2]));
}
