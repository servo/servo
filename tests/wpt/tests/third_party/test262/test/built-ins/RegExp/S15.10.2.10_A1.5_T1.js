// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterEscape :: r evaluates by returning
    the character \u000D
es5id: 15.10.2.10_A1.5_T1
description: Use \r in RegExp and \u000D in tested string
---*/

var arr = /\r/.exec("\u000D");
if ((arr === null) || (arr[0] !== "\u000D")) {
  throw new Test262Error('#1: var arr = /\\r/.exec("\\u000D"); arr[0] === "\\u000D". Actual. ' + (arr && arr[0]));
}

var arr = /\r\r/.exec("a\u000D\u000Db");
if ((arr === null) || (arr[0] !== "\u000D\u000D")) {
  throw new Test262Error('#2: var arr = /\\r\\r/.exec("a\\u000D\\u000Db"); arr[0] === "\\u000D\\u000D". Actual. ' + (arr && arr[0]));
}
