// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalEscape :: DecimalIntegerLiteral [lookahead not in DecimalDigit]"
es5id: 15.10.2.11_A1_T1
description: >
    DecimalEscape :: 0. If i is zero, return the EscapeValue
    consisting of a <NUL> character (Unicodevalue0000)
---*/

var arr = /\0/.exec("\u0000"); 
if ((arr === null) || (arr[0] !== "\u0000")) {
  throw new Test262Error('#1: var arr = /\\0/.exec(\\u0000); arr[0] === "\\u0000". Actual. ' + (arr && arr[0]));
}

var arr = (new RegExp("\\0")).exec("\u0000"); 
if ((arr === null) || (arr[0] !== "\u0000")) {
  throw new Test262Error('#2: var arr = (new RegExp("\\0")).exec(\\u0000); arr[0] === "\\u0000". Actual. ' + (arr && arr[0]));
}
