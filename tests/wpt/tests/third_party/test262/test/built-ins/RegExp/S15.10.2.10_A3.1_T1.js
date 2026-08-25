// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "CharacterEscape :: HexEscapeSequence :: x HexDigit HexDigit"
es5id: 15.10.2.10_A3.1_T1
description: Tested string include equal unicode symbols
---*/

var arr = /\x00/.exec("\u0000"); 
if ((arr === null) || (arr[0] !== "\u0000")) {
  throw new Test262Error('#0: var arr = /\\x00/.exec(\\u0000); arr[0] === "\\u0000". Actual. ' + (arr && arr[0]));
}

var arr = /\x01/.exec("\u0001"); 
if ((arr === null) || (arr[0] !== "\u0001")) {
  throw new Test262Error('#1: var arr = /\\x01/.exec(\\u0001); arr[0] === "\\u0001". Actual. ' + (arr && arr[0]));
}

var arr = /\x0A/.exec("\u000A"); 
if ((arr === null) || (arr[0] !== "\u000A")) {
  throw new Test262Error('#2: var arr = /\\x0A/.exec(\\u000A); arr[0] === "\\u000A". Actual. ' + (arr && arr[0]));
}

var arr = /\xFF/.exec("\u00FF"); 
if ((arr === null) || (arr[0] !== "\u00FF")) {
  throw new Test262Error('#3: var arr = /\\xFF/.exec(\\u00FF); arr[0] === "\\u00FF". Actual. ' + (arr && arr[0]));
}
