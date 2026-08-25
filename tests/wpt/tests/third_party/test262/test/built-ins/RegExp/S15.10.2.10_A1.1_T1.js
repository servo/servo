// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterEscape :: t evaluates by returning
    the character \u0009
es5id: 15.10.2.10_A1.1_T1
description: Use \t in RegExp and \u0009 in tested string
---*/

var arr = /\t/.exec("\u0009");
if ((arr === null) || (arr[0] !== "\u0009")) {
  throw new Test262Error('#1: var arr = /\\t/.exec("\\u0009"); arr[0] === "\\u0009". Actual. ' + (arr && arr[0]));
}

var arr = /\t\t/.exec("a\u0009\u0009b");
if ((arr === null) || (arr[0] !== "\u0009\u0009")) {
  throw new Test262Error('#2: var arr = /\\t\\t/.exec("a\\u0009\\u0009b"); arr[0] === "\\u0009\\u0009". Actual. ' + (arr && arr[0]));
}
